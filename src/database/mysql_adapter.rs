use std::borrow::BorrowMut;
use std::time::Duration;

use mysql::serde::de::Error;
use mysql::{prelude::*, TxOpts};
use serde_yaml::Error as YamlError;

use crate::masker::transformer::Options;
use crate::masker::{self, PkType};

use super::adapter::DatabaseAdapter;

pub struct MySQLAdapter {
    connection_creds: MySQLConnectionCredentials,
}

impl MySQLAdapter {
    pub fn new_from_yaml(yaml: &serde_yaml::Value) -> Result<Self, Box<dyn std::error::Error>> {
        let connection_creds = MySQLConnectionCredentials::from_yaml(yaml)?;
        Ok(MySQLAdapter { connection_creds })
    }

    fn verify_entities(
        &self,
        masker: &masker::Masker,
        conn: &mut mysql::PooledConn,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let db_tables: Vec<String> = conn.query("SHOW TABLES;")?;

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
    ) -> Result<String, Box<dyn std::error::Error>> {
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

    fn get_batch_to_update(
        &self,
        masker_entity: &masker::Entity,
        conn: &mut mysql::PooledConn,
        b_size: u32,
        offset: u32,
    ) -> Result<Vec<String>, mysql::error::Error> {
        let values: Vec<usize> = conn.exec(
            format!(
                "SELECT {} FROM {} ORDER BY {} ASC LIMIT ? OFFSET ?",
                masker_entity.get_pk_name(),
                masker_entity.get_table_name(),
                masker_entity.get_pk_name()
            ),
            (b_size, offset),
        )?;
        Ok(values.iter().map(|v| v.to_string()).collect())
    }

    fn mask_table(
        &self,
        masker_entity: &masker::Entity,
        mut conn: mysql::PooledConn,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let b_size = 30;
        let mut offs_idx = 0;
        loop {
            let ids = self.get_batch_to_update(
                masker_entity,
                conn.borrow_mut(),
                b_size,
                offs_idx * b_size,
            )?;
            if ids.is_empty() {
                break;
            }
            let mut tr = conn.start_transaction(TxOpts::default())?;
            for id in ids {
                _ = tr.query::<String, String>(
                    self.prepare_entity_query(masker_entity, Box::new(id))?,
                );
            }
            tr.commit()?;
            offs_idx += 1;
        }
        Ok(())
    }
}

impl DatabaseAdapter for MySQLAdapter {
    fn apply_mask(&self, masker: &crate::masker::Masker) -> Result<(), Box<dyn std::error::Error>> {
        let con_pool_opts = mysql::Opts::from_url(self.connection_creds.get_as_string().as_str())?;
        let pool = match mysql::Pool::new(con_pool_opts) {
            Ok(p) => p,
            Err(e) => return Err(e.into()),
        };
        let mut c = pool.try_get_conn(Duration::new(30, 0))?;
        self.verify_entities(masker, &mut c)?;
        for entity in masker.get_entities() {
            let pool = pool.clone();
            let c = pool.try_get_conn(Duration::new(30, 0))?;
            self.mask_table(entity, c)?;
        }
        Ok(())
    }
}

struct MySQLConnectionCredentials {
    host: String,
    username: String,
    password: String,
    db_name: String,
    port: String,
}

impl MySQLConnectionCredentials {
    pub fn new(
        host: String,
        username: String,
        password: String,
        db_name: String,
        port: String,
    ) -> Self {
        Self {
            host,
            username,
            password,
            port,
            db_name,
        }
    }
    pub fn from_yaml(
        yaml: &serde_yaml::Value,
    ) -> Result<MySQLConnectionCredentials, serde_yaml::Error> {
        match yaml["connection"].as_mapping() {
            Some(m) => {
                let host = m
                    .get("host")
                    .ok_or(YamlError::missing_field(
                        "Couldn't find 'host' in DB connection credentials",
                    ))?
                    .as_str()
                    .ok_or(YamlError::custom("Couldn't convert host to string"))?
                    .to_string();
                let username = m
                    .get("username")
                    .ok_or(YamlError::missing_field(
                        "Couldn't find 'username' in DB connection credentials.",
                    ))?
                    .as_str()
                    .ok_or(YamlError::custom("Couldn't convert username to string."))?
                    .to_string();
                let password = m
                    .get("password")
                    .ok_or(YamlError::missing_field(
                        "Couldn't find 'password' in DB connection credentials.",
                    ))?
                    .as_str()
                    .ok_or(YamlError::custom("Couldn't convert password to string."))?
                    .to_string();
                let db_name = m
                    .get("db_name")
                    .ok_or(YamlError::missing_field(
                        "Couldn't find 'db_name' in DB connection credentials.",
                    ))?
                    .as_str()
                    .ok_or(YamlError::custom("Couldn't convert db_name to string."))?
                    .to_string();
                let port_node = m.get("port").ok_or(YamlError::missing_field(
                    "Couldn't find 'port' in DB connection credentials.",
                ))?;
                let port = match port_node.as_i64() {
                    Some(p_i64) => p_i64.to_string(),
                    None => match port_node.as_str() {
                        Some(p_str) => p_str.to_string(),
                        None => return Err(YamlError::custom("Couldn't convert port to string.")),
                    },
                };
                Ok(Self::new(host, username, password, db_name, port))
            }
            None => Err(YamlError::missing_field(
                "Couldn't find connection credentials at .connection in the yaml.",
            )),
        }
    }

    pub fn get_as_string(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.db_name
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::masker::{transformer::FirstNameTransformer, Entity, Field};

    use super::*;

    fn get_test_conn() -> mysql::Pool {
        let con_pool_opts =
            mysql::Opts::from_url("mysql://root:root@localhost/classicmodels").unwrap();
        mysql::Pool::new(con_pool_opts).unwrap()
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

    #[test]
    fn adapter_masks_table_values() {
        let adapter = get_adapter();
        let t_name = "customers";
        let pk_name = "customerNumber";
        let field_name = "contactFirstName";
        let fields: Vec<Field> = vec![Field::new(
            field_name.to_string(),
            Box::new(FirstNameTransformer {}),
        )];
        let entity = Entity::new(t_name.to_string(), pk_name.to_string(), PkType::Int, fields);
        let pc = get_test_conn();
        let before = pc
            .get_conn()
            .unwrap()
            .query::<String, String>(format!(
                "SELECT {} FROM {} ORDER BY {} ASC LIMIT 1",
                field_name, t_name, pk_name
            ))
            .unwrap();
        let c = pc.get_conn().unwrap();
        adapter.mask_table(&entity, c).unwrap();
        let after = pc
            .get_conn()
            .unwrap()
            .query::<String, String>(format!(
                "SELECT {} FROM {} ORDER BY {} ASC LIMIT 1",
                field_name, t_name, pk_name
            ))
            .unwrap();

        assert_ne!(before, after)
    }

    #[test]
    fn adapter_get_batch_to_update_returns_ids() {
        let adapter = get_adapter();
        let t_name = "customers";
        let pk_name = "customerNumber";
        let fields: Vec<Field> = vec![];
        let entity = Entity::new(t_name.to_string(), pk_name.to_string(), PkType::Int, fields);
        let pc = get_test_conn();
        let mut con = pc.get_conn().unwrap();

        let expected = vec!["103", "112", "114", "119", "121"];
        let res = adapter
            .get_batch_to_update(&entity, &mut con, 5, 0)
            .unwrap();

        assert_eq!(res, expected)
    }
}
