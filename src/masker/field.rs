use std::fmt::Display;

use crate::masker::transformer::{new_from_yaml, Options, Transformer};

use super::transformer::GeneratedValue;

pub struct Field {
    field_name: String,
    transformer: Box<dyn Transformer>,
}

impl Field {
    pub fn new(field_name: String, transformer: Box<dyn Transformer>) -> Self {
        Self {
            field_name,
            transformer,
        }
    }

    pub fn generate(&self, opts: &Options) -> Result<GeneratedValue, Box<dyn std::error::Error>> {
        self.transformer.generate(opts)
    }

    pub fn new_from_yaml(yaml: &serde_yaml::Value) -> Result<Self, Box<dyn std::error::Error>> {
        let name =
            match yaml["name"].as_str() {
                Some(s) => String::from(s),
                None => return Err(
                    "Tried to read entry from a fields list, but couldn't locate 'name' property"
                        .into(),
                ),
            };
        let transformer = new_from_yaml(yaml).unwrap();
        Ok(Self::new(name, transformer))
    }

    pub fn get_column_name(&self) -> &String {
        &self.field_name
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "field_name: {}", self.field_name)
    }
}
