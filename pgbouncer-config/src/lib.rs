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
//!     let config_text = config.expr();
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
#[cfg(feature = "io")]
pub mod io;
pub mod utils;