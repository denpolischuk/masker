use std::fmt::Display;

use crate::masker::transformer::{new_from_yaml, Options, Transformer};

pub enum SupportedSchemaEntries {
    FirstName,
    LastName,
    Template,
    MobilePhone,
}

impl Display for SupportedSchemaEntries {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportedSchemaEntries::FirstName => write!(f, "FirstName"),
            SupportedSchemaEntries::LastName => write!(f, "LastName"),
            SupportedSchemaEntries::Template => write!(f, "Template"),
            SupportedSchemaEntries::MobilePhone => write!(f, "MobilePhone"),
        }
    }
}

pub struct SchemaEntry {
    field_name: String,
    transformer: Box<dyn Transformer>,
}

impl SchemaEntry {
    pub fn new(field_name: String, transformer: Box<dyn Transformer>) -> Self {
        Self {
            field_name,
            transformer,
        }
    }

    pub fn generate(&self, opts: Options) -> Result<mysql::Value, Box<dyn std::error::Error>> {
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

impl Display for SchemaEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "field_name: {}", self.field_name)
    }
}
