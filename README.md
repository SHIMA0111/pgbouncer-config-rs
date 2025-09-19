# PgBouncer Config Rust

A Rust workspace to safely generate, modify, and compare PgBouncer configuration files (pgbouncer.ini) from both a library and a CLI.

This workspace consists of two crates:

- pgbouncer-config: A library to model, build, parse, diff, and read/write PgBouncer settings (the contents of pgbouncer.ini) in a type-safe way
- pgbouncer-generator: A CLI powered by the above library. It can generate pgbouncer.ini from an intermediate definition file (TOML/JSON) and show diffs


## Features

- Strongly typed handling of PgBouncer settings ([pgbouncer] and [databases] sections) in Rust
- Generate pgbouncer.ini text and parse an existing pgbouncer.ini back to structures
- Read and write structured intermediate definition files in TOML/JSON
- Import database lists from multiple PostgreSQL hosts
- Compute and display differences between the current pgbouncer.ini and an intermediate definition as JSON
- Execute a full workflow from the CLI (init, add template, add DB, import, diff, generate)


## Workspace layout

- pgbouncer-config: Library crate (edition = 2024)
- pgbouncer-generator: CLI crate (edition = 2024)
- generated/: Sample intermediate definition files and the pgbouncer.ini output location

See Cargo.toml (workspace) for workspace metadata.


## Requirements

- Rust 1.80+ (recommend a stable toolchain that supports edition 2024)
- Domain knowledge of PgBouncer (setting names and meanings)
- Appropriate network access/credentials if you connect to PostgreSQL hosts to gather info


## Build and Test

- Build the whole workspace
  - cargo build
- Run library tests
  - cargo test -p pgbouncer-config


## Install (CLI only)

Install pgbouncer-generator from this workspace locally:

- cargo install --path pgbouncer-generator

You can also run without installing via cargo run.


## Usage (CLI: pgbouncer-generator)

The CLI provides the following subcommands. See src/main.rs for default arguments.

- init: Create an initial intermediate definition file (TOML)
- add-empty-pg-template: Add an empty Postgres template to the definition
- add-pg: Add a single Postgres entry (host, port, credentials, database list, etc.) to the definition
- import: Import database names from the specified Postgres hosts into the definition
- diff: Show the JSON diff between the current pgbouncer.ini and the definition
- generate: Generate pgbouncer.ini from the definition

Default paths

- Definition: ./generated/pgbouncer_definition.toml
- Output pgbouncer.ini: ./generated/pgbouncer.ini

Examples

1) Initialize (create the definition)

- cargo run -p pgbouncer-generator -- init

Overwrite if the file already exists

- cargo run -p pgbouncer-generator -- init --force-overwrite

2) Add an empty Postgres template

- cargo run -p pgbouncer-generator -- add-empty-pg-template

3) Add a Postgres entry (user/password, target DBs, ignore DBs, whether to output credentials in config, etc.)

- cargo run -p pgbouncer-generator -- add-pg \
    --host localhost \
    --port 5432 \
    --user postgres \
    --password postgres \
    --databases db1 db2 db3 \
    --ignore-databases template0 template1 \
    --is-output-credentials-to-config false \
    --allow-not-exist true

4) Import DB names from Postgres hosts

- cargo run -p pgbouncer-generator -- import \
    --target-postgres-host 10.0.0.10 10.0.0.11

5) Show diff (compare the definition with the current pgbouncer.ini)

- cargo run -p pgbouncer-generator -- diff \
    --path-def-file ./generated/pgbouncer_definition.toml \
    --path-pgbouncer-ini ./generated/pgbouncer.ini

6) Generate (create pgbouncer.ini from the definition)

- cargo run -p pgbouncer-generator -- generate

If you do not want to overwrite, pass --allow-overwrite false.


## Library usage (pgbouncer-config)

Dependency (Cargo.toml)

```toml
[dependencies]
pgbouncer-config = { version = "0.1", git = "https://github.com/SHIMA0111/pgbouncer-config-rs" }
# or use a local path
# pgbouncer-config = { path = "../pgbouncer-config" }
```

Sample code

```rust,no_run
use pgbouncer_config::builder::PgBouncerConfigBuilder;
use pgbouncer_config::pgbouncer_config::databases_setting::{Database, DatabasesSetting};
use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
use pgbouncer_config::pgbouncer_config::PgBouncerConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build the [databases] section
    let mut db_setting = DatabasesSetting::new();
    let db = Database::new("localhost", 5432, "postgres", "postgres", None);
    db_setting.add_database(db);

    // Build the [pgbouncer] section
    let mut pgbouncer_setting = PgBouncerSetting::default();
    pgbouncer_setting.set_listen_addr("0.0.0.0");

    // Build the full config
    let config: PgBouncerConfig = PgBouncerConfigBuilder::builder()
        .set_databases_setting(db_setting)?
        .set_pgbouncer_setting(pgbouncer_setting)?
        .build();

    // To INI string
    let ini_text = config.expr();
    println!("{}", ini_text);

    Ok(())
}
```

I/O

- Read INI

```rust,ignore
use std::io::Cursor;
use pgbouncer_config::io::read::Reader;

let ini = "[pgbouncer]\nlisten_addr = 127.0.0.1\nlisten_port = 6432\nauth_type = md5\nmax_client_conn = 100\ndefault_pool_size = 20\npool_mode = session\n";
let mut reader = Reader::new(Cursor::new(ini.as_bytes()));
let cfg = reader.read()?; // parse pgbouncer.ini into the struct
```

- Write INI/TOML/JSON

```rust,ignore
use pgbouncer_config::io::{write::Writer, ConfigFileFormat};
use std::fs::File;
use std::path::Path;

let mut writer_ini = Writer::try_from(pgbouncer_config::io::write::Writers::File(Path::new("./generated/pgbouncer.ini")))?;
writer_ini.write(&cfg)?; // write as INI

let mut writer_toml = Writer::try_from(pgbouncer_config::io::write::Writers::File(Path::new("./generated/pgbouncer_definition.toml")))?;
writer_toml.write_config(&cfg, ConfigFileFormat::TOML)?; // write as TOML
```

Diff calculation

- Compute the difference between the current pgbouncer.ini and the intermediate definition (TOML/JSON) and print it as JSON (see the diff subcommand).


## License

- See LICENSE-mit.md and LICENSE-ap.md bundled with this repository (dual-licensed under Apache-2.0 / MIT) for the library/tool license terms.


## Contributing

- Issues/PRs are welcome. Please follow standard Rust code style. 
- Tests are welcome as well (especially around I/O and diff processing).


## Acknowledgments

- Thanks to the PgBouncer project and the Rust community.

---

Language

- Japanese README is available at [README in Japanese](README-ja.md).