use crate::masker::Field;
use std::{borrow::Borrow, fmt::Display, vec};

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

    pub fn new_from_yaml(yaml: &serde_yaml::Value) -> Result<Self, Box<dyn std::error::Error>> {
        let s_name = match yaml["table"].as_str() {
            Some(s) => String::from(s),
            None => return Err("Missing or invalid 'table' entry in YAML file".into()),
        };

        let s_pk_name = match yaml["pk"]["name"].as_str() {
            Some(s) => String::from(s),
            None => return Err("Missing or invalid 'pk.type' entry in YAML file. You should specify primary key name and type [int string]".into()),
        };

        let s_pk_type = match yaml["pk"]["type"].as_str() {
            Some(s) => match s {
                "int" => PkType::Int,
                "string" => PkType::String,
                other => return Err(format!("Unknown primary key type 'pk.type' {}", other).into()),
            }
            None => return Err("Missing or invalid 'pk.type' entry in YAML file. You should specify primary key name and type [int string]".into()),
        };

        let mut s_entries: Vec<Field> = vec::Vec::new();
        match yaml["fields"].as_sequence() {
            Some(seq) => seq
                .iter()
                .for_each(|entry| s_entries.push(Field::new_from_yaml(entry).unwrap())), // It's
            // ok to unwrap here, because we want to panic on corrupted yaml schema
            None => {
                return Err(
                    "Missing or invalid 'fields' entry in the YAML file. Nothing to map".into(),
                )
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
