use mysql::serde::de::Error;
use serde_yaml::Error as YamlError;

use crate::masker::Masker;

use super::mysql_adapter::MySQLAdapter;
pub trait DatabaseAdapter {
    fn apply_mask(&self, masker: &Masker) -> Result<(), Box<dyn std::error::Error>>;
}

pub fn new_db_adapter_from_yaml(
    yaml: &serde_yaml::Value,
) -> Result<Box<dyn DatabaseAdapter>, Box<dyn std::error::Error>> {
    match yaml["db"].as_mapping() {
        Some(m) => match m.get("family") {
            Some(f) => match f.as_str() {
                Some(f_str) => match f_str {
                    // Add adapters here
                    "mysql" => Ok(Box::new(MySQLAdapter::new_from_yaml(&yaml["db"])?)),
                    us => Err(format!("Unknown db family {}", us).into()),
                },
                None => Err("Expected but couldn't find 'family' class in 'db' config".into()),
            },
            None => Err(YamlError::missing_field(
                "Expected but couldn't find 'family' class in 'db' config",
            )
            .into()),
        },
        None => Err(YamlError::missing_field(
            "Couldn't find 'db' object in config yaml. Please provide db data",
        )
        .into()),
    }
}
