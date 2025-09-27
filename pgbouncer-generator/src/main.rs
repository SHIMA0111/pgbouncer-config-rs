mod formatter;

use std::path::Path;
use clap::{Parser, Subcommand};
use pgbouncer_config::builder::PgBouncerConfigBuilder;
use pgbouncer_config::io::ConfigFileFormat::TOML;
use pgbouncer_config::io::read::{Reader, Readers};
use pgbouncer_config::io::write::{Writer, Writers};
use pgbouncer_config::pgbouncer_config::databases_setting::{Database, DatabasesSetting};
use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
use pgbouncer_config::pgbouncer_config::PgBouncerConfig;
use pgbouncer_config::utils::diff::{compute_diff_pg_config};

#[derive(Parser, Debug)]
struct Cli {
    #[clap(subcommand)]
    command: Commands
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Generate the empty definition file to generate pgbouncer.ini file")]
    Init {
        #[clap(
            help = "The path of the intermediate definition file to generate",
            short,
            long,
            default_value = "./generated/pgbouncer_definition.toml",
        )]
        path_def_file: String,
        #[clap(
            help = "Enabling SSH tunnel setting to the definition file",
            short,
            long,
            default_value = "false",
        )]
        enable_ssh_tunnel: bool,
        #[clap(
            help = "Flag if the definition file should be overwritten if it exists",
            short,
            long,
            default_value = "false",
        )]
        force_overwrite: bool,
    },
    #[command(about = "Add a new postgres template to the definition file")]
    AddEmptyPgTemplate {
        #[clap(
            help = "The path of the intermediate definition file",
            short,
            long,
            default_value = "./generated/pgbouncer_definition.toml",
        )]
        path_def_file: String,
        #[clap(
            help = "Enabling SSH tunnel setting to the definition file",
            short,
            long,
            default_value = "false",
        )]
        enable_ssh_tunnel: bool,
        #[clap(
            help = "Allow to create a new definition file if the definition file does not exist when add empty Postgres template",
            short,
            long,
            default_value = "false",
        )]
        allow_not_exist: bool,
    },
    #[command(about = "Add a postgres information to the definition file")]
    AddPg {
        #[clap(
            help = "The path of the intermediate definition file",
            short,
            long,
            default_value = "./generated/pgbouncer_definition.toml",
        )]
        path_def_file: String,
        #[clap(
            help = "The host of the Postgres (unsupported <host>:<port> format)",
            short = 'd',
            long,
            default_value = "localhost",
        )]
        host: String,
        #[clap(
            help = "The port of the Postgres",
            short = 'n',
            long,
            default_value = "5432",
        )]
        port: u16,
        #[clap(
            help = "The user of the Postgres",
            short,
            long,
            default_value = "postgres",
        )]
        user: String,
        #[clap(
            help = "The password of the Postgres",
            short = 'i',
            long,
            default_value = "postgres",
        )]
        password: String,
        #[clap(
            help = "The databases in the Postgres",
            long,
            value_parser,
            value_delimiter = ' ',
            num_args = 1..,
        )]
        databases: Vec<String>,
        #[clap(
            help = "The databases in the Postgres to ignore when use PgBouncer",
            long,
            value_parser,
            value_delimiter = ' ',
            num_args = 1..,
        )]
        ignore_databases: Vec<String>,
        #[clap(
            help = "True if the user/password should be output each databases section in pgbouncer.ini file",
            short = 'c',
            long,
            default_value = "false",
        )]
        is_output_credentials_to_config: bool,
        #[clap(
            help = "Allow to create a new definition file if the definition file does not exist when add Postgres",
            short,
            long,
            default_value = "false",
        )]
        allow_not_exist: bool,
    },
    #[command(about = "Import databases from the Postgres host")]
    Import {
        #[clap(
            help = "The path of the intermediate definition file",
            short,
            long,
            default_value = "./generated/pgbouncer_definition.toml",
        )]
        path_def_file: String,
        #[clap(
            help = "The hosts of the Postgres to import databases",
            short,
            long,
            value_parser,
            value_delimiter = ' ',
            num_args = 1..,
        )]
        target_postgres_host: Vec<String>,
    },
    #[command(about = "Display the difference between definition file and current pgbouncer.ini file")]
    Diff {
        #[clap(
            help = "The path of the intermediate definition file",
            short,
            long,
            default_value = "./generated/pgbouncer_definition.toml",
        )]
        path_def_file: String,
        #[clap(
            help = "The path of the pgbouncer.ini file",
            short = 'c',
            long,
            default_value = "./generated/pgbouncer.ini",
        )]
        path_pgbouncer_ini: String,
        #[clap(
            help = "Flag if decorate the diff output or not",
            short,
            long,
            default_value = "false",
        )]
        disable_decorated_output: bool,
        #[clap(
            help = "The maximum depth of the diff output if this specified as 0, the diff output will be unlimited",
            short,
            long,
            default_value = "0",
        )]
        max_diff_depth: usize,
        #[clap(
            help = "Flag if show the same value in the diff output or not",
            short,
            long,
            default_value = "false",
        )]
        show_same: bool,
    },
    #[command(about = "Generate pgbouncer.ini file from the definition file")]
    Generate {
        #[clap(
            help = "The path of the intermediate definition file",
            short = 'd',
            long,
            default_value = "./generated/pgbouncer_definition.toml",
        )]
        path_def_file: String,
        #[clap(
            help = "The path of the pgbouncer.ini file",
            short = 'c',
            long,
            default_value = "./generated/pgbouncer.ini",
        )]
        path_pgbouncer_ini: String,
        #[clap(
            help = "Allow to overwrite the pgbouncer.ini file if it exists",
            short = 'o',
            long,
            default_value = "false",
        )]
        disallow_overwrite: bool,
    },
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Init { path_def_file, enable_ssh_tunnel, force_overwrite } => {
            let path: &Path = path_def_file.as_str().as_ref();
            if path.exists() && !force_overwrite {
                return Err(anyhow::anyhow!("The definition file already exists"));
            }

            let pgbouncer_setting = PgBouncerSetting::default();
            let mut db_setting = DatabasesSetting::new();
            if enable_ssh_tunnel {
                db_setting.add_empty_database_with_tunnel();
            } else {
                db_setting.add_empty_database();
            }

            let pgbouncer_config = PgBouncerConfigBuilder::builder()
                .set_pgbouncer_setting(pgbouncer_setting)?
                .set_databases_setting(db_setting)?
                .build();

            let mut writer = Writer::try_from(Writers::File(path))?;
            writer.write_config(&pgbouncer_config, TOML)?;

            Ok(())
        },
        Commands::AddEmptyPgTemplate { path_def_file, enable_ssh_tunnel, allow_not_exist } => {
            let path: &Path = path_def_file.as_str().as_ref();
            let mut current_setting = load_config_from_definition(path, allow_not_exist)?;

            let db_setting = current_setting.get_config_mut::<DatabasesSetting>()?;
            if enable_ssh_tunnel {
                db_setting.add_empty_database_with_tunnel();
            } else {
                db_setting.add_empty_database();
            }
            let mut writer = Writer::try_from(Writers::File(path))?;
            writer.write_config(&current_setting, TOML)?;

            Ok(())
        },
        Commands::AddPg {
            path_def_file,
            host,
            port,
            user,
            password,
            databases,
            ignore_databases,
            is_output_credentials_to_config,
            allow_not_exist
        } => {
            let path: &Path = path_def_file.as_str().as_ref();
            let mut current_setting = load_config_from_definition(path, allow_not_exist)?;

            let mut database = Database::new(&host, port, &user, &password, get_option_vec_str(&databases).as_deref());
            for ignore_database in ignore_databases.iter() {
                database.add_ignore_database(ignore_database.as_str());
            }
            database.set_is_output_credentials_to_config(is_output_credentials_to_config);

            let db_setting = current_setting.get_config_mut::<DatabasesSetting>()?;
            db_setting.add_database(database);

            let mut writer = Writer::try_from(Writers::File(path))?;
            writer.write_config(&current_setting, TOML)?;

            Ok(())
        },
        Commands::Import { path_def_file, target_postgres_host } => {
            let path: &Path = path_def_file.as_str().as_ref();
            let mut current_setting = load_config_from_definition(path, false)?;

            let db_setting = current_setting.get_config_mut::<DatabasesSetting>()?;

            db_setting.add_database_from_hosts(get_option_vec_str(&target_postgres_host).as_deref()).await?;

            let mut writer = Writer::try_from(Writers::File(path))?;
            writer.write_config(&current_setting, TOML)?;

            Ok(())
        },
        Commands::Diff {
            path_def_file,
            path_pgbouncer_ini,
            disable_decorated_output,
            max_diff_depth,
            show_same
        } => {
            let path: &Path = path_def_file.as_str().as_ref();
            let path_pgbouncer_ini: &Path = path_pgbouncer_ini.as_str().as_ref();

            let definition = load_config_from_definition(path, false)?;
            let current_ini = load_config_from_ini(path_pgbouncer_ini)?;

            let diff = compute_diff_pg_config(&current_ini, &definition)?;

            let opts = formatter::DisplayOptions::new(
                !disable_decorated_output,
                show_same,
                max_diff_depth
            );
            let formatted_diff = formatter::format_diff(&diff, opts);
            println!("{}", formatted_diff);

            Ok(())
        },
        Commands::Generate { path_def_file, path_pgbouncer_ini, disallow_overwrite } => {
            let path: &Path = path_def_file.as_str().as_ref();
            let path_pgbouncer_ini: &Path = path_pgbouncer_ini.as_str().as_ref();
            if path_pgbouncer_ini.exists() && disallow_overwrite {
                return Err(anyhow::anyhow!("The pgbouncer.ini file already exists, if you want to overwrite it, please use the --allow-overwrite option"));
            }

            let definition = load_config_from_definition(path, false)?;
            let mut writer = Writer::try_from(Writers::File(path_pgbouncer_ini))?;
            writer.write(&definition)?;

            Ok(())
        }
    }
}

fn load_config_from_definition(path: &Path, allow_not_exist: bool) -> anyhow::Result<PgBouncerConfig> {
    if !path.exists() && !allow_not_exist {
        return Err(anyhow::anyhow!("The definition file does not exist and allow_not_exist is false"));
    }

    let current_setting = if path.exists() {
        Reader::try_from(Readers::File(path))?.read_config(TOML)?
    } else {
        let pgbouncer_setting = PgBouncerSetting::default();
        let db_setting = DatabasesSetting::new();
        PgBouncerConfigBuilder::builder()
            .set_pgbouncer_setting(pgbouncer_setting)?
            .set_databases_setting(db_setting)?
            .build()
    };

    Ok(current_setting)
}

fn load_config_from_ini(path: &Path) -> anyhow::Result<PgBouncerConfig> {
    if !path.exists() {
        return Err(anyhow::anyhow!("The pgbouncer.ini file does not exist"));
    }

    let mut reader = Reader::try_from(Readers::File(path))?;
    let pgbouncer_ini = reader.read()?;

    Ok(pgbouncer_ini)
}

fn get_option_vec_str(value: &[String]) -> Option<Vec<&str>> {
    if value.is_empty() {
        None
    } else {
        Some(value.iter().map(|s| s.as_str()).collect::<Vec<_>>())
    }
}