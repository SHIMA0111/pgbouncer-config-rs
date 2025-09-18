use std::collections::HashMap;
use std::ops::Index;
use std::sync::Arc;
use futures::future::join_all;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::error::PgBouncerError;
use crate::pg_client::PgClient;
use crate::pgbouncer_config::Expression;
use crate::utils::parser::{parse_key_value, ParserIniFromStr};

/// Databases section settings.
///
/// Represents the [databases] section of pgbouncer-config.ini. Use this to manage a
/// collection of Database entries and render them into configuration text.
///
/// # Fields
/// - databases: List of backend database routing entries.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DatabasesSetting {
    databases: Vec<Database>,
}

impl DatabasesSetting {
    /// Create an empty DatabasesSetting.
    ///
    /// # Returns
    /// The initialized DatabasesSetting with no databases.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::databases_setting::DatabasesSetting;
    /// let settings = DatabasesSetting::new();
    /// ```
    pub fn new() -> Self {
        Self {
            databases: vec![],
        }
    }

    /// Add a Database entry to the collection.
    ///
    /// # Parameters
    /// - database: The Database to append.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::databases_setting::{DatabasesSetting, Database};
    /// let mut settings = DatabasesSetting::new();
    /// let db = Database::default();
    /// settings.add_database(db);
    /// ```
    pub fn add_database(&mut self, database: Database) -> Self {
        let mut same_databases = self.databases
            .iter()
            .filter(|&db|
                db.host == database.host() &&
                db.port == database.port() &&
                db.user == database.user() &&
                db.password == database.password())
            .map(|db| db.clone())
            .collect::<Vec<Database>>();

        if same_databases.len() > 0 {
            same_databases.push(database);
            let new_db = Self::merge_databases(same_databases);
            self.databases.push(new_db);
        }
        else {
            self.databases.push(database);
        }

        self.clone()
    }

    /// Add a default Database entry.
    ///
    /// Inserts a Database::default() into the collection.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::databases_setting::DatabasesSetting;
    /// let mut settings = DatabasesSetting::new();
    /// settings.add_empty_database();
    /// ```
    pub fn add_empty_database(&mut self) -> Self {
        let database = Database::default();
        self.add_database(database);

        self.clone()
    }

    /// Fetches databases from PostgreSQL hosts for the contained `Database` entries concurrently.
    ///
    /// For each `Database` in this setting, this method asynchronously calls
    /// [`Database::get_databases_from_host`] with `None` as the argument (one task per
    /// entry), optionally filtering by the provided host list.
    ///
    /// # Parameters
    /// - target_hosts: Optional list of host names to target. If `None` or empty,
    ///   all `Database` entries are processed.
    ///
    /// # Returns
    /// Unit on success.
    ///
    /// # Errors
    /// Returns an error if any spawned task fails to join or if any
    /// [`Database::get_databases_from_host`] call returns an error.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use pgbouncer_config::pgbouncer_config::databases_setting::{DatabasesSetting, Database};
    ///
    /// // Build a setting with one host and fetch its databases asynchronously.
    /// let mut settings = DatabasesSetting::new();
    /// settings.add_database(Database::new("127.0.0.1", 5432, "postgres", "postgres", None));
    ///
    /// let rt = tokio::runtime::Runtime::new().unwrap();
    /// rt.block_on(async {
    ///     // Process all hosts
    ///     settings.add_database_from_hosts(None).await.unwrap();
    ///
    ///     // Or only specific hosts
    ///     settings.add_database_from_hosts(Some(&vec!["127.0.0.1"]))
    ///         .await
    ///         .unwrap();
    /// });
    /// ```
    ///
    /// # Notes
    /// - Requires a Tokio runtime.
    /// - Spawns one task per `Database` entry and waits for all to complete.
    /// - Internally clones each `Database` before fetching.
    pub async fn add_database_from_hosts(&self, target_hosts: Option<&[&str]>) -> crate::error::Result<()> {
        let hosts = if let Some(hosts) = target_hosts {
            hosts.iter().map(|&host| host.to_string()).collect()
        } else {
            vec![]
        };

        let mut temp_db_joins = vec![];
        for database in &self.databases {
            if hosts.len() > 0 && !hosts.contains(&database.host().to_string()) {
                continue;
            }

            let temp_db = Arc::new(Mutex::new(database.clone()));
            let temp_db_clone = temp_db.clone();
            temp_db_joins.push(tokio::spawn(async move {
                let mut temp_db_lock = temp_db_clone.lock().await;
                temp_db_lock.get_databases_from_host(None).await
            }));
        }

        let join_reses = join_all(temp_db_joins).await;
        for join_res in join_reses {
            join_res??;
        }

        Ok(())
    }

    fn merge_databases(mut databases: Vec<Database>) -> Database {
        let mut database = databases.remove(0);
        for db in databases {
            database.push_databases(&db.databases);
        }

        database
    }
}

impl Default for DatabasesSetting {
    fn default() -> Self {
        Self::new()
    }
}

/// Index into the DatabasesSetting by position.
///
/// # Parameters
/// - index: The position of the Database to retrieve.
///
/// # Panics
/// Panics if the index is out of bounds.
impl Index<usize> for DatabasesSetting {
    type Output = Database;

    fn index(&self, index: usize) -> &Self::Output {
        &self.databases[index]
    }
}

#[typetag::serde]
impl Expression for DatabasesSetting {
    /// Render the [databases] section as configuration text.
    ///
    /// Concatenates the expr() of each contained Database, prefixed with
    /// a [databases] header.
    ///
    /// # Returns
    /// The configuration text for the [databases] section.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::databases_setting::{DatabasesSetting, Database};
    /// use pgbouncer_config::pgbouncer_config::Expression;
    ///
    /// let mut settings = DatabasesSetting::new();
    /// settings.add_database(Database::default());
    /// let text = settings.expr();
    /// assert!(text.starts_with("[databases]\n"));
    /// ```
    fn expr(&self) -> String {
        let mut expr = "[databases]\n".to_string();

        expr.push_str(
            &self.databases
                .iter()
                .map(|database| database.expr())
                .collect::<Vec<String>>()
                .join("\n")
        );

        expr
    }

    fn config_section_name(&self) -> &'static str {
        "DataBaseInfo"
    }
}

impl ParserIniFromStr for DatabasesSetting {
    type Error = PgBouncerError;

    fn parse_from_str(value: &str) -> Result<Self, Self::Error> {
        let mut database_setting = DatabasesSetting::new();
        for value_line in value.trim().split("\n") {
            let database = Database::parse_from_str(value_line)?;
            database_setting.add_database(database);
        }

        Ok(database_setting)
    }
}

/// A single database routing entry.
///
/// Represents how PgBouncer should connect to a backend PostgreSQL instance and
/// which logical databases to expose through this route.
///
/// # Fields
/// - host: Backend PostgreSQL host.
/// - port: Backend PostgreSQL port.
/// - user: Username used when embedding credentials in the config output.
/// - password: Password used when embedding credentials in the config output.
/// - databases: Logical database names this route will expose.
/// - ignore_databases: Database names to exclude when rendering.
/// - is_output_credentials_to_config: If true, embed user/password into the
///   generated config lines. Defaults to false.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Database {
    host: String,
    port: u16,
    user: String,
    password: String,
    databases: Vec<String>,
    ignore_databases: Vec<String>,
    is_output_credentials_to_config: bool,
}

impl Database {
    /// Create a new Database routing entry.
    ///
    /// # Parameters
    /// - host: Backend PostgreSQL host.
    /// - port: Backend PostgreSQL port.
    /// - user: Username for the backend (used if credentials are embedded).
    /// - password: Password for the backend (used if credentials are embedded).
    /// - databases: List of logical database names to expose via this route.
    ///
    /// # Returns
    /// The initialized Database entry.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::databases_setting::Database;
    /// let db = Database::new("127.0.0.1", 5432, "postgres", "postgres", Some(&vec!["app"]));
    /// ```
    pub fn new(
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        databases: Option<&[&str]>,
    ) -> Self {
        let databases = match databases {
            Some(databases) => databases
                .iter()
                .map(|&db| db.to_string())
                .collect(),
            None => vec![],
        };
        Self {
            host: host.to_string(),
            port,
            user: user.to_string(),
            password: password.to_string(),
            databases,
            ignore_databases: vec![],
            is_output_credentials_to_config: false,
        }
    }

    /// Extend the databases list with additional names.
    ///
    /// Duplicates are removed and the list is kept sorted.
    ///
    /// # Parameters
    /// - databases: Slice of database names to add.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::databases_setting::Database;
    /// let mut db = Database::default();
    /// db.push_databases(&vec!["a".to_string(), "b".to_string(), "a".to_string()]);
    /// ```
    pub fn push_databases(&mut self, databases: &[String]) -> Self {
        self.databases.extend(databases.iter().cloned());
        self.databases.sort();
        self.databases.dedup();

        self.clone()
    }
    
    /// Set the backend host.
    ///
    /// # Parameters
    /// - host: Hostname or IP address of the PostgreSQL server.
    ///
    /// # Returns
    /// The updated configuration with the new host.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::databases_setting::Database;
    /// let mut db = Database::default();
    /// let db2 = db.set_host("db.internal");
    /// ```
    pub fn set_host(&mut self, host: &str) -> Self {
        self.host = host.to_string();
        self.clone()
    }
    
    /// Set the backend port.
    ///
    /// # Parameters
    /// - port: TCP port of the PostgreSQL server.
    ///
    /// # Returns
    /// The updated configuration with the new port.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::databases_setting::Database;
    /// let mut db = Database::default();
    /// let db2 = db.set_port(5433);
    /// ```
    pub fn set_port(&mut self, port: u16) -> Self {
        self.port = port;
        self.clone()
    }
    
    /// Set the backend user name.
    ///
    /// # Parameters
    /// - user: User to connect as when credentials are embedded.
    ///
    /// # Returns
    /// The updated configuration with the new user.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::databases_setting::Database;
    /// let mut db = Database::default();
    /// let db2 = db.set_user("app");
    /// ```
    pub fn set_user(&mut self, user: &str) -> Self {
        self.user = user.to_string();
        self.clone()
    }
    
    /// Set the backend password.
    ///
    /// # Parameters
    /// - password: Password to use when credentials are embedded.
    ///
    /// # Returns
    /// The updated configuration with the new password.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::databases_setting::Database;
    /// let mut db = Database::default();
    /// let db2 = db.set_password("secret");
    /// ```
    pub fn set_password(&mut self, password: &str) -> Self {
        self.password = password.to_string();
        self.clone()
    }
    
    /// Add a logical database name to expose.
    ///
    /// Deduplicates and keeps the list sorted.
    ///
    /// # Parameters
    /// - database: Database name to add.
    ///
    /// # Returns
    /// The updated configuration with the database added.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::databases_setting::Database;
    /// let mut db = Database::default();
    /// let db2 = db.add_database("analytics");
    /// ```
    pub fn add_database(&mut self, database: &str) -> Self {
        self.databases.push(database.to_string());
        self.databases.sort();
        self.databases.dedup();
        self.clone()
    }
    
    /// Exclude a database name from the rendered output.
    ///
    /// Deduplicates and keeps the ignore list sorted.
    ///
    /// # Parameters
    /// - database: Database name to exclude.
    ///
    /// # Returns
    /// The updated configuration with the database excluded.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::databases_setting::Database;
    /// let mut db = Database::default();
    /// let db2 = db.add_ignore_database("template0");
    /// ```
    pub fn add_ignore_database(&mut self, database: &str) -> Self {
        self.ignore_databases.push(database.to_string());
        self.ignore_databases.sort();
        self.ignore_databases.dedup();
        self.clone()
    }
    
    /// Control whether credentials are embedded into the generated config.
    ///
    /// When set to true, expr() will include "user" and "password" key-value
    /// pairs. This may be convenient but has security implications since
    /// credentials end up in plain text configuration output.
    ///
    /// # Parameters
    /// - is_output_credentials_to_config: Whether to embed credentials.
    ///
    /// # Returns
    /// The updated configuration reflecting the new setting.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::databases_setting::Database;
    /// let mut db = Database::default();
    /// let db2 = db.set_is_output_credentials_to_config(true);
    /// ```
    pub fn set_is_output_credentials_to_config(&mut self, is_output_credentials_to_config: bool) -> Self {
        self.is_output_credentials_to_config = is_output_credentials_to_config;
        self.clone()
    }

    /// Asynchronously retrieves a list of databases from a specified PostgreSQL host and updates the internal state.
    ///
    /// # Parameters
    /// - `default_db`: An optional reference to a string slice specifying the default database to connect to.
    ///   If not provided, the function defaults to using the "postgres" database.
    ///
    /// # Errors
    /// - Returns an error (wrapped in `crate::error::Result`) if any of the following operations fail:
    ///   - Establishing a connection using `PgClient::new`.
    ///   - Fetching databases using `client.get_databases`.
    ///
    /// # Returns
    /// - Returns `Ok(())` on success, indicating that the database list was successfully updated.
    pub async fn get_databases_from_host(&mut self, default_db: Option<&str>) -> crate::error::Result<()> {
        let db_name = default_db.unwrap_or("postgres");

        let client = PgClient::new(
            self.host(),
            self.port(),
            self.user(),
            self.password(),
            db_name,
        ).await?;
        let db_names = client.get_databases().await?;
        self.push_databases(&db_names);

        Ok(())
    }

    /// Render this Database as one or more configuration lines.
    ///
    /// For each logical database in `databases` that is not present in
    /// `ignore_databases`, a line in the form
    /// `name = dbname=name host=HOST port=PORT [user=USER password=PASS]`
    /// is emitted. Credentials are included only when
    /// `is_output_credentials_to_config` is true.
    ///
    /// # Returns
    /// Configuration lines terminated by newlines. May be empty if all
    /// databases are ignored.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::databases_setting::Database;
    /// use pgbouncer_config::pgbouncer_config::Expression;
    ///
    /// let mut db = Database::default();
    /// let text = db.expr();
    /// assert!(text.contains("host=127.0.0.1"));
    /// ```
    pub fn expr(&self) -> String {
        let mut expr = String::new();

        for database in &self.databases {
            if self.ignore_databases.contains(database) {
                continue;
            }

            let mut line = String::new();

            line.push_str(&format!(
                "{0} = dbname={0} host={1} port={2}",
                database, self.host, self.port
            ));

            if self.is_output_credentials_to_config {
                line.push_str(&format!(" user = {}", self.user));
                line.push_str(&format!(" password = {}", self.password));
            }

            expr.push_str(&format!("{}\n", line));
        }

        expr
    }

    fn host(&self) -> &str {
        &self.host
    }

    fn port(&self) -> u16 {
        self.port
    }

    fn user(&self) -> &str {
        &self.user
    }

    fn password(&self) -> &str {
        &self.password
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new(
            "127.0.0.1", 
            5432, 
            "postgres", 
            "postgres", 
            Some(&vec![
                "postgres"
            ])
        )
    }
}

impl ParserIniFromStr for Database {
    type Error = PgBouncerError;

    fn parse_from_str(value: &str) -> Result<Self, Self::Error> {
        let (_, body) = parse_key_value(value)?;

        let pair_re = Regex::new(
            r#"(?x)(?P<k>\w+)=(?P<v> '(?:[^'\\]|\\.)*'| "(?:[^"\\]|\\.)*"| \S+)"#,
        )?;

        let mut map: HashMap<String, String> = HashMap::new();
        for cap in pair_re.captures_iter(&body) {
            let k = cap.name("k").ok_or(
                PgBouncerError::PgBouncer(format!("Invalid argument key: {}", value))
            )?.as_str().to_string();
            let v = cap.name("v").ok_or(
                PgBouncerError::PgBouncer(format!("Invalid argument value: {}", value))
            )?.as_str().to_string();
            map.insert(k, v);
        }

        let dbname = map.remove("dbname").ok_or(
            PgBouncerError::PgBouncer(format!("Not found 'dbname': {}", value))
        )?;
        let host = map.remove("host").ok_or(
            PgBouncerError::PgBouncer(format!("Not found 'host': {}", value))
        )?;
        let port: u16 = map
            .remove("port")
            .ok_or(
                PgBouncerError::PgBouncer(format!("Not found 'port': {}", value))
            )?
            .parse()
            .map_err(|_| PgBouncerError::PgBouncer(format!("Invalid port: {}", value)))?;

        let user = map.remove("user");
        let password = map.remove("password");
        let db_names = vec![dbname.as_str()];

        Ok(Database::new(
            &host,
            port,
            user.as_deref().unwrap_or("<nothing>"),
            password.as_deref().unwrap_or("<nothing>"),
            Some(&db_names),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn databases_setting_expr_starts_with_header() {
        let settings = DatabasesSetting::new();
        let text = settings.expr();
        assert!(text.starts_with("[databases]\n"));
    }

    #[test]
    fn database_expr_includes_host_port_and_optional_credentials() {
        let mut db = Database::new(
            "10.0.0.1", 15432, "user", "pass", Some(&vec!["app"]));
        // Without credentials output
        let text = db.expr();
        assert!(text.contains("dbname=app"));
        assert!(text.contains("host=10.0.0.1"));
        assert!(text.contains("port=15432"));
        assert!(!text.contains("user = user"));
        assert!(!text.contains("password = pass"));

        // With credentials output
        db = db.set_is_output_credentials_to_config(true);
        let text2 = db.expr();
        assert!(text2.contains("user = user"));
        assert!(text2.contains("password = pass"));
    }

    #[test]
    fn database_parse_from_str_parses_one_line() {
        let line = "app = dbname=app host=127.0.0.1 port=5432 user=postgres password=postgres";
        let db = Database::parse_from_str(line).expect("parse line");
        let out = db.expr();
        assert!(out.contains("dbname=app"));
        assert!(out.contains("host=127.0.0.1"));
        assert!(out.contains("port=5432"));
    }

    #[test]
    fn push_databases_dedups_and_sorts() {
        let mut db = Database::new("127.0.0.1", 5432, "u", "p", Some(&vec!["b", "a"]));
        db.push_databases(&vec!["a".to_string(), "c".to_string(), "b".to_string()]);
        // expr contains a, b, c lines once each
        let text = db.expr();
        let count_a = text.lines().filter(|l| l.starts_with("a = ")).count();
        let count_b = text.lines().filter(|l| l.starts_with("b = ")).count();
        let count_c = text.lines().filter(|l| l.starts_with("c = ")).count();
        assert_eq!(count_a, 1);
        assert_eq!(count_b, 1);
        assert_eq!(count_c, 1);
    }
}
