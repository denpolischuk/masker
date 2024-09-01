use mysql::prelude::Queryable;

use crate::masker::SchemaEntry;
use std::{fmt::Display, vec};

pub struct Schema {
    name: String,
    pk_name: String,
    pub entries: Vec<SchemaEntry>,
}

impl Schema {
    fn new(name: String, pk_name: String, entries: Vec<SchemaEntry>) -> Self {
        Self {
            name,
            pk_name,
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

        let mut s_entries: Vec<SchemaEntry> = vec::Vec::new();
        match yaml["fields"].as_sequence() {
            Some(seq) => seq
                .iter()
                .for_each(|entry| s_entries.push(SchemaEntry::new_from_yaml(entry).unwrap())), // It's
            // ok to unwrap here, because we want to panic on corrupted yaml schema
            None => {
                return Err(
                    "Missing or invalid 'fields' entry in the YAML file. Nothing to map".into(),
                )
            }
        };

        Ok(Schema::new(s_name, s_pk_name, s_entries))
    }

    pub fn get_table_name(&self) -> String {
        self.name.to_string()
    }

    pub fn mask(&self, mut c: mysql::PooledConn) -> Result<(), Box<dyn std::error::Error>> {
        let q = format!(
            "SELECT {} from {} ORDER BY {} ASC LIMIT {}, {}",
            self.pk_name, self.name, self.pk_name, "0", "30"
        );
        println!("query = {q}");
        let queries: Vec<String> =
            c.query_map::<Box<str>, _, String, String>(q, |pk| format!("UPDATE {}", pk))?;

        println!("{:?}", queries);
        Ok(())
    }
}

impl Display for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "name: {}, pk_name: {}", self.name, self.pk_name)?;
        writeln!(f, "entries: \n")?;
        for entry in &self.entries {
            writeln!(f, " {}", entry)?;
        }
        Ok(())
    }
}
