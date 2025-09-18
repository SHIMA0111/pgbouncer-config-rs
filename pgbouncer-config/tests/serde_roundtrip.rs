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
".to_string()
}

#[test]
fn ini_to_json_and_back() {
    let ini = minimal_ini();
    let mut r_ini = Reader::new(Cursor::new(ini.as_bytes()));
    let cfg = r_ini.read().expect("parse ini");

    let mut buf: Vec<u8> = Vec::new();
    let mut w = Writer::new(&mut buf);
    w.write_config(&cfg, ConfigFileFormat::JSON).expect("write json");

    let json = String::from_utf8(buf).expect("utf8");
    let mut r_json = Reader::new(Cursor::new(json.as_bytes()));
    let cfg2 = r_json.read_config(ConfigFileFormat::JSON).expect("read json");

    // ensure same serialization
    assert_eq!(serde_json::to_string(&cfg).unwrap(), serde_json::to_string(&cfg2).unwrap());
}

#[test]
fn ini_to_toml_and_back() {
    let ini = minimal_ini();
    let mut r_ini = Reader::new(Cursor::new(ini.as_bytes()));
    let cfg = r_ini.read().expect("parse ini");

    let mut buf: Vec<u8> = Vec::new();
    let mut w = Writer::new(&mut buf);
    w.write_config(&cfg, ConfigFileFormat::TOML).expect("write toml");

    let toml_s = String::from_utf8(buf).expect("utf8");
    let mut r_toml = Reader::new(Cursor::new(toml_s.as_bytes()));
    let cfg2 = r_toml.read_config(ConfigFileFormat::TOML).expect("read toml");

    // ensure same serialization
    assert_eq!(toml::to_string(&cfg).unwrap(), toml::to_string(&cfg2).unwrap());
}
