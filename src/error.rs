use thiserror::Error;


#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] DbError),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Serialization/Deserialization error: {0}")]
    Serialization(String),

    #[error("Background task panicked")]
    TaskPanic,

    #[error("Tokio task error: {0}")]
    TokioTask(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[derive(Error, Debug, Clone)]
pub enum DbError {
    #[error("Database schema migration failed: {0}")]
    MigrationFailed(String),

    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Unique constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Record not found")]
    NotFound,

    #[error("Query execution failed: {0}")]
    QueryFailed(String),
}

impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DbError::NotFound,
            sqlx::Error::Database(db_err) => {
                let msg = db_err.message().to_string();
                if db_err.is_unique_violation() {
                    DbError::ConstraintViolation(msg)
                } else {
                    DbError::QueryFailed(msg)
                }
            }
            sqlx::Error::Io(_) => DbError::ConnectionFailed("IO Error".to_string()),
            e => DbError::QueryFailed(e.to_string()),
        }
    }
}
