use crate::masker::error::ConfigParseError;

pub fn parse_runtime_env_values(
    yaml: &serde_yaml::Value,
) -> Result<Option<String>, ConfigParseError> {
    let field = "fromEnvKey";
    match yaml.as_mapping() {
        Some(mp) => match mp.get(field) {
            Some(v) => match v.as_str() {
                Some(key) => std::env::var(key).map(Some).map_err(|e| ConfigParseError {
                    kind: crate::masker::error::ConfigParseErrorKind::FailedToReadValueFromEnv(
                        String::from(key),
                        e,
                    ),
                    field: String::from(field),
                }),
                None => Err(ConfigParseError {
                    kind: crate::masker::error::ConfigParseErrorKind::UnexpectedFieldType,
                    field: String::from(field),
                }),
            },
            None => Err(ConfigParseError {
                kind: crate::masker::error::ConfigParseErrorKind::UnexpectedFieldType,
                field: String::from(field),
            }),
        },
        None => Ok(None),
    }
}

pub fn read_str_field(
    yaml: &serde_yaml::Mapping,
    field: String,
) -> Result<String, ConfigParseError> {
    let node = yaml.get(&field).ok_or(ConfigParseError {
        kind: crate::masker::error::ConfigParseErrorKind::MissingField,
        field: field.clone(),
    })?;
    match parse_runtime_env_values(node)? {
        Some(val) => Ok(val),
        None => Ok(node
            .as_str()
            .ok_or(ConfigParseError {
                kind: crate::masker::error::ConfigParseErrorKind::UnexpectedFieldType,
                field,
            })?
            .to_string()),
    }
}

pub fn read_str_or_int_field(
    yaml: &serde_yaml::Mapping,
    field: String,
) -> Result<String, ConfigParseError> {
    let node = yaml.get(&field).ok_or(ConfigParseError {
        kind: crate::masker::error::ConfigParseErrorKind::MissingField,
        field: field.clone(),
    })?;
    match parse_runtime_env_values(node)? {
        Some(val) => Ok(val),
        None => Ok(match node.as_i64() {
            Some(p_i64) => p_i64.to_string(),
            None => match node.as_str() {
                Some(p_str) => p_str.to_string(),
                None => {
                    return Err(ConfigParseError {
                        kind: crate::masker::error::ConfigParseErrorKind::UnexpectedFieldType,
                        field,
                    })
                }
            },
        }),
    }
}
