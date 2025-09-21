#[cfg(feature = "io")]
mod tests {
    use pgbouncer_config::io::{read::Reader, write::Writer, ConfigFileFormat};
    use std::io::Cursor;

    fn minimal_ini() -> String {
        "\
    [pgbouncer]\n\
    listen_addr = 127.0.0.1\n\
    listen_port = 6432\n\
    auth_type = md5\n\
    max_client_conn = 100\n\
    default_pool_size = 20\n\
    pool_mode = session\n\
    \n\
    [databases]\n\
    app = dbname=app host=127.0.0.1 port=5432\n\
    ".to_string()
    }

    #[test]
    fn ini_to_ini_roundtrip() {
        let ini = minimal_ini();
        let mut r_ini = Reader::new(Cursor::new(ini.as_bytes()));
        let cfg = r_ini.read().expect("parse ini");

        // Write back to INI
        let mut buf: Vec<u8> = Vec::new();
        let mut w = Writer::new(&mut buf);
        w.write(&cfg).expect("write ini");
        assert!(!buf.is_empty());

        // Read again (trim trailing whitespace) and ensure stable serialization via JSON equality
        let ini2 = String::from_utf8(buf).expect("utf8");
        let ini2_trimmed = ini2.trim().to_string();
        let mut r2 = Reader::new(Cursor::new(ini2_trimmed.as_bytes()));
        let cfg2 = r2.read().expect("read ini");
        assert_eq!(
            serde_json::to_string(&cfg).unwrap(),
            serde_json::to_string(&cfg2).unwrap()
        );
    }

    #[test]
    fn write_read_consistency_across_formats() {
        let ini = minimal_ini();
        let mut r_ini = Reader::new(Cursor::new(ini.as_bytes()));
        let cfg = r_ini.read().expect("parse ini");

        // JSON roundtrip using Writer then Reader
        let mut json_buf: Vec<u8> = Vec::new();
        let mut w_json = Writer::new(&mut json_buf);
        w_json
            .write_config(&cfg, ConfigFileFormat::JSON)
            .expect("write json");
        let mut r_json = Reader::new(Cursor::new(json_buf.as_slice()));
        let cfg_json = r_json
            .read_config(ConfigFileFormat::JSON)
            .expect("read json");
        assert_eq!(
            serde_json::to_string(&cfg).unwrap(),
            serde_json::to_string(&cfg_json).unwrap()
        );

        // TOML roundtrip using Writer then Reader
        let mut toml_buf: Vec<u8> = Vec::new();
        let mut w_toml = Writer::new(&mut toml_buf);
        w_toml
            .write_config(&cfg, ConfigFileFormat::TOML)
            .expect("write toml");
        let mut r_toml = Reader::new(Cursor::new(toml_buf.as_slice()));
        let cfg_toml = r_toml
            .read_config(ConfigFileFormat::TOML)
            .expect("read toml");
        assert_eq!(
            toml::to_string(&cfg).unwrap(),
            toml::to_string(&cfg_toml).unwrap()
        );
    }
}
