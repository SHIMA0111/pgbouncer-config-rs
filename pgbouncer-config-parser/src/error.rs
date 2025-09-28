use std::fmt::{Display, Formatter};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, Error)]
pub struct ParseError {
    reason: String,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error: {}", self.reason)
    }
}