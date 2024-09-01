use crate::masker::Entity;

pub struct Masker {
    connection_creds: DatabaseConnectionCredentials,
    pub entities: Vec<Entity>,
}

impl Masker {
    pub fn new_from_yaml(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let f = std::fs::File::open(filename)?;
        let data = serde_yaml::from_reader(f)?;
        let creds = DatabaseConnectionCredentials::new_from_yaml(&data)?;
        let mut schemas: Vec<Entity> = vec![];
        match data["schemas"].as_sequence() {
            Some(seq) => {
                seq.iter()
                    .try_for_each(|schema_yaml| -> Result<(), Box<dyn std::error::Error>> {
                        schemas.push(Entity::new_from_yaml(schema_yaml)?);
                        Ok(())
                    })
            }
            None => return Err("Missing schemas definition in yaml.".into()),
        }?;
        Ok(Masker {
            connection_creds: creds,
            entities: schemas,
        })
    }

    pub fn get_conn_str(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.connection_creds.username,
            self.connection_creds.password,
            self.connection_creds.host,
            self.connection_creds.port,
            self.connection_creds.db_name
        )
    }
}

pub struct DatabaseConnectionCredentials {
    host: String,
    username: String,
    password: String,
    db_name: String,
    port: String,
}

impl DatabaseConnectionCredentials {
    pub fn new_from_yaml(yaml: &serde_yaml::Value) -> Result<Self, Box<dyn std::error::Error>> {
        match yaml["connection"].as_mapping() {
            Some(m) => {
                let host = m
                    .get("host")
                    .ok_or("Couldn't find 'host' in DB connection credentials.")?
                    .as_str()
                    .ok_or("Couldn't convert host to string.")?
                    .to_string();
                let username = m
                    .get("username")
                    .ok_or("Couldn't find 'username' in DB connection credentials.")?
                    .as_str()
                    .ok_or("Couldn't convert username to string.")?
                    .to_string();
                let password = m
                    .get("password")
                    .ok_or("Couldn't find 'password' in DB connection credentials.")?
                    .as_str()
                    .ok_or("Couldn't convert password to string.")?
                    .to_string();
                let db_name = m
                    .get("db_name")
                    .ok_or("Couldn't find 'db_name' in DB connection credentials.")?
                    .as_str()
                    .ok_or("Couldn't convert db_name to string.")?
                    .to_string();
                let port_node = m
                    .get("port")
                    .ok_or("Couldn't find 'port' in DB connection credentials.")?;
                let port = match port_node.as_i64() {
                    Some(p_i64) => p_i64.to_string(),
                    None => match port_node.as_str() {
                        Some(p_str) => p_str.to_string(),
                        None => return Err("Couldn't convert port to string.".into()),
                    },
                };
                Ok(DatabaseConnectionCredentials {
                    host,
                    username,
                    password,
                    db_name,
                    port,
                })
            }
            None => Err("Couldn't find connection credentials at .connection in the yaml.".into()),
        }
    }
}
