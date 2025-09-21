//! # PgBouncer Config
//!
//! A Rust library for building pgbouncer-config(a.k.a. pgbouncer.ini) and wrapping utilization
//! for generating pgbouncer.ini files.
//!
//! ## Features
//!
//! - **Building pgbouncer config file** - PgBouncer config can be generated programmatically
//! - **Parsing PgBouncer config file** - Can parse pgbouncer.ini file to the PgBouncer config structure
//! - **Generate intermediate setting file** - To generate pgbouncer.ini file you can use intermediate file
//!   formatted TOML/JSON
//! - **Import databases from PostgreSQL host** - Import all databases from Postgres host
//! - **Return difference between two config/setting** - Retrieves the difference between 2 configs/settings
//!
//! ## Quick Start
//! Add this crate to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! pgbouncer-config = "0.1.0"
//! ```
//!
//! ## Basic Usage
//!
//! ```rust
//! use pgbouncer_config::builder::PgBouncerConfigBuilder;
//! use pgbouncer_config::pgbouncer_config::databases_setting::{Database, DatabasesSetting};
//! use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
//! use pgbouncer_config::pgbouncer_config::{PgBouncerConfig, Expression};
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut db_setting = DatabasesSetting::new();
//!     let mut db = Database::new("localhost", 5432, "postgres", "postgres", None);
//!
//!     db_setting.add_database(db);
//!
//!     let mut pgbouncer_setting = PgBouncerSetting::default();
//!     pgbouncer_setting.set_listen_addr("pgbouncer-config.example");
//!
//!     let config: PgBouncerConfig = PgBouncerConfigBuilder::builder()
//!         .set_databases_setting(db_setting).unwrap()
//!         .set_pgbouncer_setting(pgbouncer_setting).unwrap()
//!         .build();
//!
//!     let config_text = config.expr().unwrap();
//!     assert!(config_text.contains("databases"));
//!     assert!(config_text.contains("pgbouncer"));
//!     assert!(config_text.contains("pgbouncer-config.example"));
//! }
//! ```
//!

pub mod pgbouncer_config;
pub mod error;
pub(crate) mod pg_client;
pub mod builder;
pub mod utils;
#[cfg(feature = "io")]
pub mod io;

#[cfg(feature = "derive")]
pub use pgbouncer_config_derive::Expression;

#[cfg(feature = "derive")]
#[allow(dead_code)]
mod __private {
    use serde::Serialize;


    pub trait ExpressionDefault: Serialize {
        fn to_expr_default(&self) -> crate::error::Result<String> {
            let mut buffer = String::new();
            let raw_value = serde_json::to_value(self)?;

            match raw_value.as_object() {
                Some(value) => {
                    for (k, v) in value {
                        if let Some(val_str) = value_to_string(v) {
                            buffer.push_str(&format!("{} = {}\n", k, val_str));
                        }
                    }
                },
                None => {}
            }

            Ok(buffer)
        }
    }

    impl <T: Serialize> ExpressionDefault for T {}

    fn value_to_string(value: &serde_json::Value) -> Option<String> {
        match value {
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Bool(b) => Some(b.to_string()),
            serde_json::Value::Array(a) => {
                let comma_separated = a.iter()
                    .map(|v| value_to_string(v))
                    .filter(|v| v.is_some())
                    // SAFETY: always Some() because of the filter above
                    .map(|v| v.unwrap())
                    .collect::<Vec<String>>()
                    .join(", ");
                Some(comma_separated)
            },
            serde_json::Value::Object(o) => {
                let comma_separated = o.iter()
                    .map(|(k, v)| {
                        let key = k.to_string();
                        let value = value_to_string(v);
                        match value {
                            Some(value) => Some(format!("{} = {}", key, value)),
                            None => None
                        }
                    })
                    .filter(|v| v.is_some())
                    .map(|v| v.unwrap())
                    .collect::<Vec<String>>()
                    .join(", ");
                Some(comma_separated)
            },
            serde_json::Value::Null => None,
        }
    }
}
