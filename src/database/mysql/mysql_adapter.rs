use super::credentials::MySQLConnectionCredentials;
use async_trait::async_trait;
use futures::future::join_all;
use sqlx::Row;
use std::sync::Arc;

use crate::database::adapter::DatabaseAdapter;
use crate::masker::transformer::Options;
use crate::masker::{self, PkType};

pub struct MySQLAdapter {
    connection_creds: MySQLConnectionCredentials,
}

impl MySQLAdapter {
    pub fn new_from_yaml(
        yaml: &serde_yaml::Value,
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let connection_creds = MySQLConnectionCredentials::from_yaml(yaml)?;
        Ok(MySQLAdapter { connection_creds })
    }

    async fn verify_entities(
        &self,
        masker: Arc<masker::Masker>,
        p: &sqlx::MySqlPool,
    ) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let rows = sqlx::query("SHOW TABLES;").fetch_all(p).await?;

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
            Err(format!("Some entities that were defined in yaml config were not found in the actual DB: {}", missing_t).into())
        }
    }

    fn prepare_entity_query(
        &self,
        masker_entity: &masker::Entity,
        id: Box<dyn ToString>,
    ) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let entity_fields = masker_entity.get_entries();
        if entity_fields.is_empty() {
            return Err(format!("Entity {} doesn't have any fields to mask. Either remove the entity from config or add fields that should be masked", masker_entity.get_table_name().as_str()).into());
        }
        let opts = Options {
            pk: Box::new(id.to_string()),
        };
        let mut values: Vec<String> = vec![];
        for entry in entity_fields {
            let val = entry.generate(&opts)?;
            let str_v = match val {
                masker::transformer::GeneratedValue::String(v) => format!("'{}'", v),
                masker::transformer::GeneratedValue::Number(v) => v,
            };
            values.push(format!("{} = {}", entry.get_column_name(), str_v));
        }
        let cond = match masker_entity.get_pk_type() {
            PkType::Int => format!("{} = {}", masker_entity.get_pk_name(), id.to_string()),
            PkType::String => {
                format!("{} = '{}'", masker_entity.get_pk_name(), id.to_string())
            }
        };
        Ok(format!(
            "UPDATE {} SET {} WHERE {}",
            masker_entity.get_table_name(),
            values.join(", "),
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
    ) -> Result<i32, Box<dyn std::error::Error + Sync + Send>> {
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
    ) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let sz_total = self.get_total_size(masker_entity, p).await?;
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
    async fn apply_mask(
        &self,
        masker: Arc<crate::masker::Masker>,
    ) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(5)
            .connect(self.connection_creds.get_as_string().as_str())
            .await?;
        self.verify_entities(masker.clone(), &pool).await?;
        let futs = masker
            .get_entities()
            .iter()
            .map(|entity| self.mask_table(entity, &pool));
        join_all(futs).await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::masker::{transformer::FirstNameTransformer, Entity, Field};

    use super::*;

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
            Field::new("name".to_string(), Box::new(FirstNameTransformer {})),
            Field::new("last_name".to_string(), Box::new(FirstNameTransformer {})),
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
            Field::new("name".to_string(), Box::new(FirstNameTransformer {})),
            Field::new("last_name".to_string(), Box::new(FirstNameTransformer {})),
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
            Box::new(FirstNameTransformer {}),
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
