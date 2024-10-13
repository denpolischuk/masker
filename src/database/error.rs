use std::fmt::Display;

use crate::masker::generator::GeneratorError;

#[derive(Debug)]
#[non_exhaustive]
pub enum DatabaseAdapterErrorKind {
    NoEntriesSpecifiedForEntity(String),
    FailedToMask(String, GeneratorError),
    QueryFailed(sqlx::Error),
    DatabaseConnectionError(sqlx::Error),
    InconsistentSchema(String),
}

#[derive(Debug)]
#[non_exhaustive]
pub struct DatabaseAdapterError {
    pub kind: DatabaseAdapterErrorKind,
}

impl Display for DatabaseAdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            DatabaseAdapterErrorKind::FailedToMask(entry, _) => write!(f, "couldn't mask {entry}"),
            DatabaseAdapterErrorKind::NoEntriesSpecifiedForEntity(entity_name) => write!(f, "entity {} doesn't have any fields to mask. Either remove the entity from config or add fields that should be masked", entity_name),
            DatabaseAdapterErrorKind::QueryFailed(e) => write!(f, "query has failed: {e}"),
            DatabaseAdapterErrorKind::DatabaseConnectionError(_) => write!(f, "connection failed"),
            DatabaseAdapterErrorKind::InconsistentSchema(missing_t) => write!(f, "some entities that were defined in yaml config were not found in the actual DB: {}", missing_t),
        }
    }
}

impl std::error::Error for DatabaseAdapterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            DatabaseAdapterErrorKind::NoEntriesSpecifiedForEntity(_) => None,
            DatabaseAdapterErrorKind::FailedToMask(_, e) => Some(e),
            DatabaseAdapterErrorKind::QueryFailed(e) => Some(e),
            DatabaseAdapterErrorKind::DatabaseConnectionError(e) => Some(e),
            DatabaseAdapterErrorKind::InconsistentSchema(_) => None,
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl DatabaseAdapterError {
    pub fn connection_error(error: sqlx::Error) -> Self {
        Self {
            kind: DatabaseAdapterErrorKind::DatabaseConnectionError(error),
        }
    }
    pub fn failed_query(error: sqlx::Error) -> Self {
        Self {
            kind: DatabaseAdapterErrorKind::QueryFailed(error),
        }
    }
    pub fn failed_to_mask(entry_name: String, generator_error: GeneratorError) -> Self {
        Self {
            kind: DatabaseAdapterErrorKind::FailedToMask(entry_name, generator_error),
        }
    }
    pub fn inconsistent_schema(missing_entity: String) -> Self {
        Self {
            kind: DatabaseAdapterErrorKind::InconsistentSchema(missing_entity),
        }
    }
}
