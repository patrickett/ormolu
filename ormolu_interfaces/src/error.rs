use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrmoluError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::error::Error),

    #[error("Other custom error: {0}")]
    Other(String),
}
