use std::fs::create_dir_all;
use std::path::Path;
use crate::io::ConfigFileFormat;
use crate::pgbouncer_config::{Expression, PgBouncerConfig};

/// Generic writer for emitting a `PgBouncerConfig` to any `std::io::Write`.
///
/// # Notes
/// - This wrapper does not own any path. It writes configuration text to the
///   inner writer as PgBouncer INI (via [`Writer::write`]) or as JSON/TOML
///   (via [`Writer::write_config`]).
pub struct Writer<W: std::io::Write>(W);

/// Output targets that can be converted into a [`Writer`].
///
/// This enum is a convenience for constructing a `Writer` from common
/// destinations like standard output, standard error, or a file path via `From`.
///
/// # Notes
/// - Constructing a [`Writer`] via `From<Writers>` will panic if the file
///   cannot be created/truncated for the [`Writers::File`] variant.
pub enum Writers<'a> {
    /// Write configuration text to standard output (stdout).
    Stdout,
    /// Write configuration text to a file at the given path (create/truncate).
    File(&'a Path),
    /// Write configuration text to standard error (stderr).
    Stderr,
}

impl<W: std::io::Write> Writer<W> {
    /// Wraps an arbitrary writer.
    ///
    /// # Parameters
    /// - writer: Any type implementing `std::io::Write` (e.g. `std::fs::File`,
    ///   `std::io::Stdout`, `std::io::Stderr`, or an in-memory `Vec<u8>`).
    ///
    /// # Returns
    /// A new `Writer` that will write to the given destination.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::io::write::Writer;
    /// let sink: Vec<u8> = Vec::new();
    /// let _w = Writer::new(sink);
    /// ```
    pub fn new(writer: W) -> Self {
        Self(writer)
    }

    /// Writes the configuration in PgBouncer INI format.
    ///
    /// This uses the `Expression` implementation to render the content that
    /// would normally appear in pgbouncer.ini.
    ///
    /// # Parameters
    /// - config: Configuration to be written.
    ///
    /// # Returns
    /// Unit on success.
    ///
    /// # Errors
    /// Returns an error if writing to the underlying writer fails.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::io::{read::Reader, write::Writer, ConfigFileFormat};
    /// use std::io::Cursor;
    ///
    /// // Build a config from INI text, then write it back out as INI
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
    /// let mut buf: Vec<u8> = Vec::new();
    /// let mut writer = Writer::new(&mut buf);
    /// writer.write(&cfg).unwrap();
    /// assert!(!buf.is_empty());
    /// ```
    pub fn write(&mut self, config: &PgBouncerConfig) -> crate::error::Result<()> {
        writeln!(self.0, "{}", config.expr())?;
        Ok(())
    }

    /// Writes the configuration serialized as JSON or TOML.
    ///
    /// Select the output format via [`ConfigFileFormat`].
    ///
    /// # Parameters
    /// - config: Configuration to be serialized.
    /// - format: Target serialization format.
    ///
    /// # Returns
    /// Unit on success.
    ///
    /// # Errors
    /// Returns an error if serialization fails or if writing fails.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::io::{read::Reader, write::Writer, ConfigFileFormat};
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
    /// let mut buf: Vec<u8> = Vec::new();
    /// let mut writer = Writer::new(&mut buf);
    /// writer.write_config(&cfg, ConfigFileFormat::JSON).unwrap();
    /// assert!(String::from_utf8(buf).unwrap().contains("PgBouncerSetting"));
    /// ```
    pub fn write_config(&mut self, config: &PgBouncerConfig, format: ConfigFileFormat) -> crate::error::Result<()> {
        let file_content = match format {
            ConfigFileFormat::JSON => {
                serde_json::to_string_pretty(config)?
            },
            ConfigFileFormat::TOML => {
                toml::to_string_pretty(config)?
            }
        };

        writeln!(self.0, "{}", file_content)?;
        Ok(())
    }
}

impl<'a> TryFrom<Writers<'a>> for Writer<Box<dyn std::io::Write>> {
    type Error = crate::error::PgBouncerError;

    fn try_from(value: Writers<'a>) -> Result<Self, Self::Error> {
        match value {
            Writers::Stdout => Ok(Self::new(Box::new(std::io::stdout()))),
            Writers::Stderr => Ok(Self::new(Box::new(std::io::stderr()))),
            Writers::File(path) => {
                if let Some(parent) = path.parent()  {
                    create_dir_all(parent)?;

                }
                let file = std::fs::File::create(path)?;
                Ok(Self::new(Box::new(file)))
            },
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
    fn writer_new_and_write_ini() {
        let ini = minimal_ini();
        // Build config from INI
        let mut reader = crate::io::read::Reader::new(Cursor::new(ini.as_bytes()));
        let cfg = reader.read().expect("parse ini");

        // Write INI to buffer
        let mut buf: Vec<u8> = Vec::new();
        let mut writer = Writer::new(&mut buf);
        writer.write(&cfg).expect("write ini");
        let text = String::from_utf8(buf).expect("utf8");
        assert!(text.contains("[pgbouncer]"));
        assert!(text.contains("listen_port = 6432"));
    }

    #[test]
    fn writer_write_config_json_and_toml() {
        let ini = minimal_ini();
        let mut reader = crate::io::read::Reader::new(Cursor::new(ini.as_bytes()));
        let cfg = reader.read().expect("parse ini");

        // JSON
        let mut buf_json: Vec<u8> = Vec::new();
        let mut writer_json = Writer::new(&mut buf_json);
        writer_json.write_config(&cfg, ConfigFileFormat::JSON).expect("write json");
        let out_json = String::from_utf8(buf_json).expect("utf8");
        // Ensure we can parse it back and it matches
        let cfg_json: crate::pgbouncer_config::PgBouncerConfig = serde_json::from_str(&out_json).expect("valid json");
        assert_eq!(serde_json::to_string(&cfg).unwrap(), serde_json::to_string(&cfg_json).unwrap());

        // TOML
        let mut buf_toml: Vec<u8> = Vec::new();
        let mut writer_toml = Writer::new(&mut buf_toml);
        writer_toml.write_config(&cfg, ConfigFileFormat::TOML).expect("write toml");
        let out_toml = String::from_utf8(buf_toml).expect("utf8");
        let cfg_toml: crate::pgbouncer_config::PgBouncerConfig = toml::from_str(&out_toml).expect("valid toml");
        assert_eq!(toml::to_string(&cfg).unwrap(), toml::to_string(&cfg_toml).unwrap());
    }
}
