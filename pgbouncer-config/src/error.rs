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
    #[error("Regex Error: {0}")]
    Regex(#[from] regex::Error),
    #[error("Tokio task Error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("SSH connection error: {0}")]
    SshConnection(#[from] russh::Error),
    #[error("SSH key error: {0}")]
    SshKey(#[from] russh::keys::Error),
    #[error("SSH authenticate error: {0}")]
    SshAuth(String),
    #[error("SSH error: {0}")]
    Connection(String),
    #[cfg(feature = "io")]
    #[error("Serialize Error: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[cfg(any(feature = "io", feature = "derive"))]
    #[error("JSON Error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[cfg(feature = "io")]
    #[error("Deserialize Error: {0}")]
    Deserialize(#[from] toml::de::Error),
}

impl Into<PgBouncerError> for String {
    fn into(self) -> PgBouncerError {
        PgBouncerError::PgBouncer(self)
    }
}

impl Into<PgBouncerError> for &str {
    fn into(self) -> PgBouncerError {
        PgBouncerError::PgBouncer(self.to_string())
    }
}