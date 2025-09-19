use thiserror::Error;

/// PgBouncer Error
pub type Result<T> = std::result::Result<T, PgBouncerError>;

#[derive(Debug, Error)]
pub enum PgBouncerError {
    #[error("SQLx Error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PgBouncer Error: {0}")]
    PgBouncer(String),
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[cfg(feature = "io")]
    #[error("Serialize Error: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[cfg(feature = "io")]
    #[error("JSON Error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[cfg(feature = "io")]
    #[error("Deserialize Error: {0}")]
    Deserialize(#[from] toml::de::Error),
    #[error("Regex Error: {0}")]
    Regex(#[from] regex::Error),
    #[error("Tokio task Error: {0}")]
    Join(#[from] tokio::task::JoinError),
}