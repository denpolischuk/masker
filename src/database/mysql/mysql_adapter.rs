use super::credentials::MySQLConnectionCredentials;
use async_trait::async_trait;
use futures::future::join_all;
use sqlx::Row;
use std::collections::HashMap;

use crate::database::adapter::DatabaseAdapter;
use crate::database::error::{DatabaseAdapterError, DatabaseAdapterErrorKind};
use crate::masker::error::ConfigParseError;
use crate::masker::generator::GeneratedValue;
use crate::masker::{self, Masker, PkType};

pub struct MySQLAdapter {
    connection_creds: MySQLConnectionCredentials,
}

impl MySQLAdapter {
    pub fn new_from_yaml(yaml: &serde_yaml::Value) -> Result<Self, ConfigParseError> {
        let connection_creds = MySQLConnectionCredentials::from_yaml(yaml)?;
        Ok(MySQLAdapter { connection_creds })
    }

    async fn verify_entities(
        &self,
        masker: &Masker,
        p: &sqlx::MySqlPool,
    ) -> Result<(), DatabaseAdapterError> {
        let rows = sqlx::query("SHOW TABLES;")
            .fetch_all(p)
            .await
            .map_err(DatabaseAdapterError::failed_query)?;

        let db_tables: Vec<String> = rows.iter().map(|r| r.get::<String, _>(0)).collect();

        let mut missing_t = String::new();
        if masker.get_entities().iter().all(|entity| -> bool {
            let t_name = entity.get_table_name();
            let check_res = db_tables.contains(&t_name);
            if !check_res {
                missing_t = t_name;
            }
            check_res
        }) {
            Ok(())
        } else {
            Err(DatabaseAdapterError::inconsistent_schema(missing_t))
        }
    }

    fn prepare_entity_query(
        &self,
        masker_entity: &masker::Entity,
        id: Box<dyn ToString>,
    ) -> Result<String, DatabaseAdapterError> {
        let entity_fields = masker_entity.get_entries();
        if entity_fields.is_empty() {
            return Err(DatabaseAdapterError {
                kind: DatabaseAdapterErrorKind::NoEntriesSpecifiedForEntity(
                    masker_entity.get_table_name(),
                ),
            });
        }
        let pk_name = masker_entity.get_pk_name();
        let id = id.to_string();
        let id_kv = match masker_entity.get_pk_type() {
            PkType::Int => (pk_name, GeneratedValue::Number(id.clone())),
            PkType::String => (pk_name, GeneratedValue::String(id.clone())),
        };
        let mut opts = HashMap::from([id_kv]);
        for entry in entity_fields {
            let val = entry.generate(&opts).map_err(|e| {
                DatabaseAdapterError::failed_to_mask(String::from(entry.get_column_name()), e)
            })?;
            opts.insert(entry.get_column_name(), val);
        }

        let id_kv = opts.remove_entry(pk_name).unwrap(); // It's important to remove id from the
                                                         // opts map before parsing it into query
        let cond = format!("{} = {}", id_kv.0, id_kv.1);

        Ok(format!(
            "UPDATE {} SET {} WHERE {}",
            masker_entity.get_table_name(),
            opts.iter()
                .map(|(k, val)| format!("{} = {}", k, val))
                .collect::<Vec<String>>()
                .join(", "),
            cond
        ))
    }

    async fn get_batch_to_update(
        &self,
        masker_entity: &masker::Entity,
        p: &sqlx::MySqlPool,
        b_size: i32,
        offset: i32,
    ) -> Result<Vec<String>, sqlx::Error> {
        let values: Vec<String> = sqlx::query(
            format!(
                "SELECT {} FROM {} ORDER BY {} ASC LIMIT ? OFFSET ?",
                masker_entity.get_pk_name(),
                masker_entity.get_table_name(),
                masker_entity.get_pk_name()
            )
            .as_str(),
        )
        .bind(b_size)
        .bind(offset)
        .fetch_all(p)
        .await?
        .iter()
        .map(|r| r.get::<i32, _>(0).to_string())
        .collect();
        Ok(values)
    }

    async fn get_total_size(
        &self,
        masker_entity: &masker::Entity,
        p: &sqlx::MySqlPool,
    ) -> Result<i32, sqlx::Error> {
        Ok(sqlx::query(
            format!(
                "SELECT COUNT({}) FROM {};",
                masker_entity.get_pk_name(),
                masker_entity.get_table_name()
            )
            .as_str(),
        )
        .fetch_one(p)
        .await?
        .get::<i32, _>(0))
    }

    async fn mask_table(
        &self,
        masker_entity: &masker::Entity,
        p: &sqlx::MySqlPool,
    ) -> Result<(), DatabaseAdapterError> {
        let sz_total = self
            .get_total_size(masker_entity, p)
            .await
            .map_err(DatabaseAdapterError::failed_query)?;
        let b_size = 1000;
        let iterations: f32 = sz_total as f32 / b_size as f32;
        let futs = (0..iterations.ceil() as i32).map(move |offs_idx| async move {
            let ids = self
                .get_batch_to_update(masker_entity, p, b_size, offs_idx * b_size)
                .await
                .unwrap();
            if ids.is_empty() {
                return;
            }
            let mut tx = p.begin().await.unwrap();
            for id in ids {
                sqlx::query(
                    self.prepare_entity_query(masker_entity, Box::new(id))
                        .unwrap()
                        .as_str(),
                )
                .execute(&mut *tx)
                .await
                .unwrap();
            }
            tx.commit().await.unwrap()
        });
        futures::future::join_all(futs).await;
        Ok(())
    }
}

#[async_trait]
impl DatabaseAdapter for MySQLAdapter {
    async fn apply_mask(&self, masker: &Masker) -> Result<(), DatabaseAdapterError> {
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(5)
            .connect(self.connection_creds.get_as_string().as_str())
            .await
            .map_err(DatabaseAdapterError::connection_error)?;
        self.verify_entities(masker, &pool).await?;
        let futs = masker
            .get_entities()
            .iter()
            .map(|entity| self.mask_table(entity, &pool));
        let res: Result<Vec<()>, DatabaseAdapterError> = join_all(futs).await.into_iter().collect();
        res.map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use fake::{faker::name::raw::*, locales::EN, Fake};
    use masker::{
        generator::{Options, SimpleGenerator},
        FieldKind,
    };

    use super::*;
    use crate::masker::{Entity, Field};

    fn get_generator() -> Box<SimpleGenerator> {
        Box::new(SimpleGenerator::new(|_: &Options| {
            Ok(GeneratedValue::String(FirstName(EN).fake::<String>()))
        }))
    }

    async fn get_test_conn() -> sqlx::Pool<sqlx::MySql> {
        sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(5)
            .connect("mysql://root:root@localhost/classicmodels")
            .await
            .unwrap()
    }

    fn get_adapter() -> MySQLAdapter {
        MySQLAdapter {
            connection_creds: MySQLConnectionCredentials::new(
                "test".to_string(),
                "username".to_string(),
                "password".to_string(),
                "db_name".to_string(),
                "port".to_string(),
            ),
        }
    }

    #[test]
    fn adapter_generates_updata_query_from_masker_entity_with_int_pk() {
        let adapter = get_adapter();
        let t_name = "table";
        let pk_name = "id";
        let fields: Vec<Field> = vec![
            Field::new("name".to_string(), FieldKind::FirstName, get_generator()),
            Field::new(
                "last_name".to_string(),
                FieldKind::LastName,
                get_generator(),
            ),
        ];
        let entity = Entity::new(t_name.to_string(), pk_name.to_string(), PkType::Int, fields);
        let id = 123;
        let query = adapter.prepare_entity_query(&entity, Box::new(id)).unwrap();
        let r = regex::Regex::new(
            r"UPDATE [A-z0-9_]+ SET [A-z_]+ = '[A-z]+', [A-z_]+ = '[A-z]+' WHERE [a-z_]+ = \d+",
        )
        .unwrap();
        assert!(r.is_match(query.as_str()))
    }

    #[test]
    fn adapter_generates_updata_query_from_masker_entity_with_string_pk() {
        let adapter = get_adapter();
        let t_name = "table";
        let pk_name = "id";
        let fields: Vec<Field> = vec![
            Field::new("name".to_string(), FieldKind::FirstName, get_generator()),
            Field::new(
                "last_name".to_string(),
                FieldKind::LastName,
                get_generator(),
            ),
        ];
        let entity = Entity::new(
            t_name.to_string(),
            pk_name.to_string(),
            PkType::String,
            fields,
        );
        let id = 123;
        let query = adapter.prepare_entity_query(&entity, Box::new(id)).unwrap();
        let r = regex::Regex::new(
            r"UPDATE [A-z0-9_]+ SET [A-z_]+ = '[A-z]+', [A-z_]+ = '[A-z]+' WHERE [a-z_]+ = '\w+'",
        )
        .unwrap();
        assert!(r.is_match(query.as_str()))
    }

    #[test]
    fn adapter_throws_error_if_entity_has_no_fields() {
        let adapter = get_adapter();
        let t_name = "table";
        let pk_name = "id";
        let fields: Vec<Field> = vec![];
        let entity = Entity::new(
            t_name.to_string(),
            pk_name.to_string(),
            PkType::String,
            fields,
        );
        let id = 123;
        assert!(adapter.prepare_entity_query(&entity, Box::new(id)).is_err());
    }

    #[tokio::test]
    async fn adapter_masks_table_values() {
        let adapter = get_adapter();
        let t_name = "customers";
        let pk_name = "customerNumber";
        let field_name = "contactFirstName";
        let fields: Vec<Field> = vec![Field::new(
            field_name.to_string(),
            FieldKind::FirstName,
            get_generator(),
        )];
        let entity = Entity::new(t_name.to_string(), pk_name.to_string(), PkType::Int, fields);
        let pool = get_test_conn().await;
        let before = sqlx::query(
            format!(
                "SELECT {} FROM {} ORDER BY {} ASC LIMIT 1",
                field_name, t_name, pk_name
            )
            .as_str(),
        )
        .fetch_one(&pool)
        .await
        .unwrap()
        .get::<String, _>(0);
        adapter.mask_table(&entity, &pool).await.unwrap();
        let after = sqlx::query(
            format!(
                "SELECT {} FROM {} ORDER BY {} ASC LIMIT 1",
                field_name, t_name, pk_name
            )
            .as_str(),
        )
        .fetch_one(&pool)
        .await
        .unwrap()
        .get::<String, _>(0);

        assert_ne!(before, after)
    }

    #[tokio::test]
    async fn adapter_get_batch_to_update_returns_ids() {
        let adapter = get_adapter();
        let t_name = "customers";
        let pk_name = "customerNumber";
        let fields: Vec<Field> = vec![];
        let entity = Entity::new(t_name.to_string(), pk_name.to_string(), PkType::Int, fields);
        let pool = get_test_conn().await;

        let expected = vec!["103", "112", "114", "119", "121"];
        let res = adapter
            .get_batch_to_update(&entity, &pool, 5, 0)
            .await
            .unwrap();

        assert_eq!(res, expected)
    }
}
