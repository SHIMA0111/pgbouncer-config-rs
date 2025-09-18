use crate::error::PgBouncerError;
use crate::io::ConfigFileFormat;
use crate::pgbouncer_config::PgBouncerConfig;
use crate::utils::parser::ParserIniFromStr;

/// Generic reader for PgBouncer configurations from any `std::io::Read`.
///
/// # Notes
/// - This wrapper does not know about file paths; it simply reads text from
///   the inner reader and parses it as PgBouncer INI (via [`Reader::read`])
///   or as JSON/TOML (via [`Reader::read_config`]).
pub struct Reader<R: std::io::Read>(R);

/// Input sources that can be converted into a [`Reader`].
///
/// This enum is a convenience for constructing a `Reader` from common
/// sources like standard input or a file path via `From`.
///
/// # Notes
/// - Constructing a [`Reader`] via `From<Readers>` will panic if the file
///   cannot be opened for the [`Readers::File`] variant.
pub enum Readers<'a> {
    /// Read configuration text from standard input (stdin).
    Stdin,
    /// Read configuration text from a file at the given path.
    File(&'a std::path::Path),
}

impl <R: std::io::Read> Reader<R> {
    /// Wraps an arbitrary reader.
    ///
    /// # Parameters
    /// - reader: Any type implementing `std::io::Read` (e.g. `std::fs::File`,
    ///   `std::io::Stdin`, or an in-memory `std::io::Cursor`).
    ///
    /// # Returns
    /// A new `Reader` that will read from the given source.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::io::read::Reader;
    /// use std::io::Cursor;
    ///
    /// let ini = "\
    /// [pgbouncer]\n\
    /// listen_addr = 127.0.0.1\n\
    /// listen_port = 6432\n\
    /// auth_type = md5\n\
    /// max_client_conn = 100\n\
    /// default_pool_size = 20\n\
    /// pool_mode = session\n\
    /// ";
    /// let mut reader = Reader::new(Cursor::new(ini.as_bytes()));
    /// let config = reader.read().unwrap();
    /// assert!(config.to_string().contains("[pgbouncer]"));
    /// ```
    pub fn new(reader: R) -> Self {
        Self(reader)
    }
    
    /// Reads all text as PgBouncer INI and parses it into `PgBouncerConfig`.
    ///
    /// # Returns
    /// Parsed `PgBouncerConfig` on success.
    ///
    /// # Errors
    /// Returns an error if reading from the underlying reader fails or if the
    /// text cannot be parsed as PgBouncer INI.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::io::read::Reader;
    /// use std::io::Cursor;
    ///
    /// let ini = "\
    /// [pgbouncer]\n\
    /// listen_addr = 127.0.0.1\n\
    /// listen_port = 6432\n\
    /// auth_type = md5\n\
    /// max_client_conn = 100\n\
    /// default_pool_size = 20\n\
    /// pool_mode = session\n\
    /// ";
    /// let mut reader = Reader::new(Cursor::new(ini.as_bytes()));
    /// let cfg = reader.read().unwrap();
    /// assert!(cfg.to_string().contains("listen_port = 6432"));
    /// ```
    pub fn read(&mut self) -> crate::error::Result<PgBouncerConfig> {
        let mut text = String::new();
        self.0.read_to_string(&mut text)?;
        Ok(PgBouncerConfig::parse_from_str(&text)?)
    }
    
    /// Reads all text and deserializes a `PgBouncerConfig` from JSON or TOML.
    ///
    /// Use [`ConfigFileFormat::JSON`] or [`ConfigFileFormat::TOML`] to choose the
    /// decoder. For PgBouncer INI format, use [`Reader::read`] instead.
    ///
    /// # Parameters
    /// - format: Which structured format to use for deserialization.
    ///
    /// # Returns
    /// Parsed `PgBouncerConfig` on success.
    ///
    /// # Errors
    /// Returns an error if reading fails or if the content cannot be
    /// deserialized from the selected format.
    pub fn read_config(&mut self, format: ConfigFileFormat) -> crate::error::Result<PgBouncerConfig> {
        let mut text = String::new();
        self.0.read_to_string(&mut text)?;
        
        let file_content = match format {
            ConfigFileFormat::JSON => {
                serde_json::from_str::<PgBouncerConfig>(&text)?
            },
            ConfigFileFormat::TOML => {
                toml::from_str::<PgBouncerConfig>(&text)?
            }
        };
        
        Ok(file_content)
    }
}

impl<'a> TryFrom<Readers<'a>> for Reader<Box<dyn std::io::Read>> {
    type Error = PgBouncerError;

    fn try_from(value: Readers<'a>) -> Result<Self, Self::Error> {
        match value {
            Readers::Stdin => Ok(Self::new(Box::new(std::io::stdin()))),
            Readers::File(path) => {
                if !path.exists() {
                    return Err(PgBouncerError::PgBouncer(format!("File not found: {}", path.display())));
                }
                Ok(Self::new(Box::new(std::fs::File::open(path)?)))
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn minimal_ini() -> String {
        let ini = "\
[pgbouncer]\n\
listen_addr = 127.0.0.1\n\
listen_port = 6432\n\
auth_type = md5\n\
max_client_conn = 100\n\
default_pool_size = 20\n\
pool_mode = session\n\
";
        ini.to_string()
    }

    #[test]
    fn reader_new_and_read_from_cursor() {
        let ini = minimal_ini();
        let mut reader = Reader::new(Cursor::new(ini.as_bytes()));
        let cfg = reader.read().expect("parse ini");
        let text = cfg.to_string();
        assert!(text.contains("[pgbouncer]"));
        assert!(text.contains("listen_port"));
    }

    #[test]
    fn reader_read_config_from_json_and_toml() {
        // Build a config via INI first, then serialize to JSON/TOML
        let ini = minimal_ini();
        let mut reader_ini = Reader::new(Cursor::new(ini.as_bytes()));
        let cfg = reader_ini.read().expect("parse ini");

        // JSON
        let json = serde_json::to_string_pretty(&cfg).expect("to json");
        let mut reader_json = Reader::new(Cursor::new(json.as_bytes()));
        let cfg_json = reader_json.read_config(ConfigFileFormat::JSON).expect("from json");
        assert_eq!(serde_json::to_string(&cfg).unwrap(), serde_json::to_string(&cfg_json).unwrap());

        // TOML
        let toml_s = toml::to_string_pretty(&cfg).expect("to toml");
        let mut reader_toml = Reader::new(Cursor::new(toml_s.as_bytes()));
        let cfg_toml = reader_toml.read_config(ConfigFileFormat::TOML).expect("from toml");
        assert_eq!(toml::to_string(&cfg).unwrap(), toml::to_string(&cfg_toml).unwrap());
    }
}
