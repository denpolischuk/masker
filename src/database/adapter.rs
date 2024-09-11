use crate::masker::{error::ConfigParseError, Masker};
use async_trait::async_trait;

use super::{error::DatabaseAdapterError, mysql::MySQLAdapter};

#[async_trait]
pub trait DatabaseAdapter {
    async fn apply_mask(&self, masker: std::sync::Arc<Masker>) -> Result<(), DatabaseAdapterError>;
}

pub fn new_db_adapter_from_yaml(
    yaml: &serde_yaml::Value,
) -> Result<Box<dyn DatabaseAdapter>, ConfigParseError> {
    match yaml["db"].as_mapping() {
        Some(m) => match m.get("family") {
            Some(f) => match f.as_str() {
                Some(f_str) => match f_str {
                    // Add adapters here
                    "mysql" => Ok(Box::new(MySQLAdapter::new_from_yaml(&yaml["db"])?)),
                    unknown_family => Err(ConfigParseError {
                        kind: crate::masker::error::ConfigParseErrorKind::UnexpectedFieldValue(
                            String::from(unknown_family),
                        ),
                        field: String::from("family"),
                    }),
                },
                None => Err(ConfigParseError {
                    kind: crate::masker::error::ConfigParseErrorKind::MissingField,
                    field: String::from("family"),
                }),
            },
            None => Err(ConfigParseError {
                kind: crate::masker::error::ConfigParseErrorKind::MissingField,
                field: String::from("family"),
            }),
        },
        None => Err(ConfigParseError {
            kind: crate::masker::error::ConfigParseErrorKind::MissingField,
            field: String::from("db"),
        }),
    }
}
