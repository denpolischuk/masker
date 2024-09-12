use crate::{database::shared, masker::error::ConfigParseError};
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
    ) -> Result<MySQLConnectionCredentials, ConfigParseError> {
        match yaml["connection"].as_mapping() {
            Some(m) => {
                let host = shared::read_str_field(m, String::from("host"))?;
                let username = shared::read_str_field(m, String::from("username"))?;
                let password = shared::read_str_field(m, String::from("password"))?;
                let db_name = shared::read_str_field(m, String::from("db_name"))?;
                let port = shared::read_str_or_int_field(m, String::from("port"))?;
                Ok(Self::new(host, username, password, db_name, port))
            }
            None => Err(ConfigParseError {
                kind: crate::masker::error::ConfigParseErrorKind::MissingField,
                field: String::from("connection"),
            }),
        }
    }

    pub fn get_as_string(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.db_name
        )
    }
}
