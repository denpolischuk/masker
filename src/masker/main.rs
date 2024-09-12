use crate::masker::Entity;

use super::error::{ConfigParseError, ConfigParseErrorKind};

pub struct Masker {
    entities: Vec<Entity>,
}

impl Masker {
    pub fn new(entities: Vec<Entity>) -> Self {
        Masker { entities }
    }

    pub fn new_from_yaml(yaml: &serde_yaml::Value) -> Result<Self, ConfigParseError> {
        let mut schemas: Vec<Entity> = vec![];
        let field = "schemas";
        match yaml[field].as_sequence() {
            Some(seq) => seq
                .iter()
                .try_for_each(|schema_yaml| -> Result<(), ConfigParseError> {
                    schemas.push(Entity::new_from_yaml(schema_yaml)?);
                    Ok(())
                }),
            None => {
                return Err(ConfigParseError {
                    kind: ConfigParseErrorKind::MissingField,
                    field: String::from(field),
                })
            }
        }?;
        Ok(Masker::new(schemas))
    }

    pub fn get_entities(&self) -> &Vec<Entity> {
        &self.entities
    }
}
