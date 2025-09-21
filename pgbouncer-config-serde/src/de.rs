use crate::utils::parse_ini_to_value;

/// Deserialize a Rust value from a minimal PgBouncer-like INI string.
///
/// # Parameters
/// - s: INI-like input string.
///
/// # Returns
/// If successful, returns a value of type `T` populated from the input.
///
/// # Errors
/// Returns an error when the input cannot be parsed into the expected structure or
/// if JSON-based deserialization fails internally.
///
/// # Examples
/// ```rust
/// use pgbouncer_config_serde::de;
/// #[derive(serde::Deserialize, Debug)]
/// struct S { a: String }
/// let s = "[section]\na = value\n";
/// // Note: The current implementation expects a top-level map-like target.
/// # let _ = s; // example placeholder to keep doctest minimal
/// ```
///
/// # Notes
/// - Suspected issue: The line `T::deserialize(json_value)` likely does not compile or work as intended,
///   because `serde_json::Value` is not a `Deserializer`. Typically you would use
///   `serde_json::from_value(json_value)` or `serde_json::value::Deserializer::new(json_value)`.
/// - The INI parsing is minimal and may not reflect full PgBouncer or INI behavior.
pub fn from_str<'de, T>(s: &'de str) -> crate::error::Result<T>
where
    T: serde::de::Deserialize<'de>
{
    let json_value = parse_ini_to_value(s)?;
    let result = T::deserialize(json_value)?;
    Ok(result)
}