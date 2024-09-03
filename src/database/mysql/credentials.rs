use mysql::serde::de::Error;
use serde_yaml::Error as YamlError;
pub struct MySQLConnectionCredentials {
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
