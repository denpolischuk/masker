use crate::masker::error::ConfigParseError;
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
                let field = String::from("host");
                let host = m
                    .get(&field)
                    .ok_or(ConfigParseError {
                        kind: crate::masker::error::ConfigParseErrorKind::MissingField,
                        field: field.clone(),
                    })?
                    .as_str()
                    .ok_or(ConfigParseError {
                        kind: crate::masker::error::ConfigParseErrorKind::UnexpectedFieldType,
                        field,
                    })?
                    .to_string();
                let field = String::from("username");
                let username = m
                    .get(&field)
                    .ok_or(ConfigParseError {
                        kind: crate::masker::error::ConfigParseErrorKind::MissingField,
                        field: field.clone(),
                    })?
                    .as_str()
                    .ok_or(ConfigParseError {
                        kind: crate::masker::error::ConfigParseErrorKind::UnexpectedFieldType,
                        field,
                    })?
                    .to_string();
                let field = String::from("password");
                let password = m
                    .get(&field)
                    .ok_or(ConfigParseError {
                        kind: crate::masker::error::ConfigParseErrorKind::MissingField,
                        field: field.clone(),
                    })?
                    .as_str()
                    .ok_or(ConfigParseError {
                        kind: crate::masker::error::ConfigParseErrorKind::UnexpectedFieldType,
                        field,
                    })?
                    .to_string();
                let field = String::from("db_name");
                let db_name = m
                    .get(&field)
                    .ok_or(ConfigParseError {
                        kind: crate::masker::error::ConfigParseErrorKind::MissingField,
                        field: field.clone(),
                    })?
                    .as_str()
                    .ok_or(ConfigParseError {
                        kind: crate::masker::error::ConfigParseErrorKind::UnexpectedFieldType,
                        field,
                    })?
                    .to_string();
                let field = String::from("port");
                let port_node = m.get(&field).ok_or(ConfigParseError {
                    kind: crate::masker::error::ConfigParseErrorKind::MissingField,
                    field: field.clone(),
                })?;
                let port =
                    match port_node.as_i64() {
                        Some(p_i64) => p_i64.to_string(),
                        None => match port_node.as_str() {
                            Some(p_str) => p_str.to_string(),
                            None => return Err(ConfigParseError {
                                kind:
                                    crate::masker::error::ConfigParseErrorKind::UnexpectedFieldType,
                                field,
                            }),
                        },
                    };
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
