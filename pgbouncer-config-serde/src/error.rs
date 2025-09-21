use std::fmt::Display;
use thiserror::Error;

/// A unified error type for the PgBouncer config serializer/deserializer.
///
/// # Variants
/// - InvalidFormat: Input did not match the expected minimal INI-like format.
/// - SerializationMessage: Generic message emitted from Serde's `Error::custom`.
/// - ComplexKey: A complex key (e.g., struct, map, tuple, or array) was attempted where a string key is required.
/// - SerdeError: Propagated error from `serde_json` used internally during deserialization.
pub type Result<T> = std::result::Result<T, PgBouncerSerdeError>;

#[derive(Debug, Error)]
pub enum PgBouncerSerdeError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Serialization error: {0}")]
    SerializationMessage(String),
    #[error("Complex types (like structs or arrays) cannot be used as INI keys)")]
    ComplexKey,
    #[error("Serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

impl serde::ser::Error for PgBouncerSerdeError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        PgBouncerSerdeError::SerializationMessage(msg.to_string())
    }
}