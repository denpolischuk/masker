use crate::masker::Field;
use std::{borrow::Borrow, fmt::Display};

use super::error::{ConfigParseError, ConfigParseErrorKind};

pub enum PkType {
    Int,
    String,
}

pub struct Entity {
    name: String,
    pk_name: String,
    pk_type: PkType,
    entries: Vec<Field>,
}

impl Entity {
    pub fn new(name: String, pk_name: String, pk_type: PkType, entries: Vec<Field>) -> Self {
        Self {
            name,
            pk_name,
            pk_type,
            entries,
        }
    }

    pub fn new_from_yaml(yaml: &serde_yaml::Value) -> Result<Self, ConfigParseError> {
        let field = String::from("table");
        let s_name = match yaml[field.as_str()].as_str() {
            Some(s) => String::from(s),
            None => {
                return Err(ConfigParseError {
                    kind: ConfigParseErrorKind::MissingField,
                    field,
                })
            }
        };

        let s_pk_name = match yaml["pk"]["name"].as_str() {
            Some(s) => String::from(s),
            None => {
                return Err(ConfigParseError {
                    kind: ConfigParseErrorKind::MissingField,
                    field: String::from("pk.name"),
                })
            }
        };

        let s_pk_type = match yaml["pk"]["type"].as_str() {
            Some(s) => match s {
                "int" => PkType::Int,
                "string" => PkType::String,
                other => {
                    return Err(ConfigParseError {
                        kind: ConfigParseErrorKind::UnexpectedFieldValue(String::from(other)),
                        field: String::from("pk.type"),
                    })
                }
            },
            None => {
                return Err(ConfigParseError {
                    kind: ConfigParseErrorKind::MissingField,
                    field: String::from("pk.type"),
                })
            }
        };

        let field = "fields";
        let s_entries: Vec<Field> = match yaml[field].as_sequence() {
            Some(seq) => seq
                .iter()
                .map(|entry| Field::new_from_yaml(entry))
                .collect::<Result<Vec<Field>, ConfigParseError>>()?,
            None => {
                return Err(ConfigParseError {
                    kind: ConfigParseErrorKind::MissingField,
                    field: String::from(field),
                })
            }
        };

        Ok(Entity::new(s_name, s_pk_name, s_pk_type, s_entries))
    }

    pub fn get_table_name(&self) -> String {
        self.name.to_string()
    }
    pub fn get_pk_name(&self) -> &String {
        self.pk_name.borrow()
    }
    pub fn get_pk_type(&self) -> &PkType {
        self.pk_type.borrow()
    }
    pub fn get_entries(&self) -> &Vec<Field> {
        self.entries.borrow()
    }
}

impl Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "name: {}, pk_name: {}", self.name, self.pk_name)?;
        writeln!(f, "entries: \n")?;
        for entry in &self.entries {
            writeln!(f, " {}", entry)?;
        }
        Ok(())
    }
}
