//! I/O utilities for reading from and writing to PgBouncer configuration files.
//!
//! This module provides generic readers and writers that operate on any
//! `std::io::Read`/`std::io::Write` implementation, along with a simple
//! file-format switch for serialization.

pub mod write;
pub mod read;

/// Configuration file formats supported by this crate when serializing/deserializing
/// a `PgBouncerConfig` from/to text.
///
/// Use this together with:
/// - `io::read::Reader::read_config` to deserialize JSON/TOML into `PgBouncerConfig`.
/// - `io::write::Writer::write_config` to serialize `PgBouncerConfig` into JSON/TOML.
pub enum ConfigFileFormat {
    /// TOML representation of `PgBouncerConfig`
    TOML,
    /// JSON representation of `PgBouncerConfig`
    JSON,
}