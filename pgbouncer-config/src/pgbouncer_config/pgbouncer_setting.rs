use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::error::PgBouncerError;
use crate::pgbouncer_config::Expression;
#[cfg(feature = "io")]
use std::collections::HashMap;
#[cfg(feature = "io")]
use crate::utils::parser::{parse_key_value, ParserIniFromStr};
#[cfg(feature = "diff")]
use crate::utils::diff::Diffable;

/// PgBouncer configuration settings.
///
/// Provides a strongly typed model for configuring PgBouncer, the PostgreSQL
/// connection pooler. Use this struct to build the [pgbouncer-config] section of a
/// pgbouncer-config.ini file via the `Expression` implementation.
///
/// # Fields
///
/// - listen_addr: IP address or host name on which PgBouncer listens.
/// - listen_port: TCP port for incoming client connections.
/// - auth_type: Authentication method (e.g., md5, scram-sha-256, trust).
/// - max_client_conn: Maximum number of allowed client connections.
/// - default_pool_size: Default number of server connections per pool.
/// - pool_mode: Pooling mode (session, transaction, or statement).
/// - admin_users: PostgreSQL users allowed to run admin commands in PgBouncer.
/// - stats_users: PostgreSQL users allowed to read statistics only.
/// - ignore_startup_parameters: Client startup parameters to ignore.
/// - logfile: Optional path to the PgBouncer log file.
/// - pidfile: Optional path to the PgBouncer PID file.
/// - auth_file: Path to the authentication file with user credentials.
/// - unix_socket_dir: Optional directory for PgBouncer Unix domain socket.
/// - auth_hba_file: Optional path to HBA configuration when using `hba` auth.
/// - auth_ident_file: Optional path to ident map file.
/// - server_check_delay: How long to keep released connections available before re-checking (seconds).
/// - server_idle_timeout: If a server connection has been idle longer than this, close it (seconds).
/// - server_lifetime: Close an unused server connection that has been connected longer than this (seconds).
/// - server_connect_timeout: Timeout for establishing server connection and login (seconds).
/// - server_login_retry: Wait time before retrying server login after failure (seconds).
/// - client_login_timeout: If a client connects but does not finish login within this time, disconnect (seconds).
/// - autodb_idle_timeout: Idle lifetime for automatically created (“*”) database pools (seconds).
/// - dns_max_ttl: Maximum TTL to cache successful DNS lookups (seconds).
/// - dns_nxdomain_ttl: TTL to cache negative DNS results (NXDOMAIN) (seconds).
/// - resolve_conf: Resolver configuration file path. If not set, use OS defaults.
/// - query_timeout: Timeout for a single query execution (seconds). 0 disables.
/// - query_wait_timeout: Timeout for waiting on a server connection from pool (seconds).
/// - cancel_wait_timeout: Timeout for forwarding CANCEL requests (seconds).
/// - client_idle_timeout: Client idle timeout (seconds). 0 disables.
/// - idle_transaction_timeout: Timeout for idle-in-transaction sessions (seconds). 0 disables.
/// - suspend_timeout: Timeout to wait for suspend to complete (seconds).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PgBouncerSetting {
    // Required settings

    /// IP address or hostname PgBouncer listens on.
    /// PgBouncer default: 127.0.0.1
    listen_addr: String,

    /// TCP port for incoming client connections.
    /// PgBouncer default: 6432
    listen_port: u16,

    /// Authentication method (e.g., md5, scram-sha-256, trust).
    /// PgBouncer default: md5
    auth_type: AuthType,

    /// Maximum number of allowed client connections.
    /// PgBouncer default: 100
    max_client_conn: u16,

    /// Default number of server connections per database/user pool.
    /// PgBouncer default: 20
    default_pool_size: u16,

    /// Pooling mode: session / transaction / statement.
    /// PgBouncer default: session
    pool_mode: PoolMode,

    /// PostgreSQL users allowed to run admin commands in PgBouncer.
    /// PgBouncer default: empty
    admin_users: Vec<String>,

    /// PostgreSQL users allowed to read statistics only.
    /// PgBouncer default: empty
    stats_users: Vec<String>,

    /// Client STARTUP parameters to ignore.
    /// PgBouncer default: empty
    ///
    /// Note: By default, PgBouncer tracks client_encoding, datestyle, timezone,
    /// standard_conforming_strings, and application_name per client.
    /// Add entries here to ignore additional startup parameters if needed.
    ignore_startup_parameters: Vec<String>,

    // Optional settings

    /// Path to the PgBouncer log file.
    /// PgBouncer default: not set
    logfile: Option<String>,

    /// Path to the PgBouncer PID file.
    /// PgBouncer default: not set
    pidfile: Option<String>,

    /// Path to the authentication file (commonly userlist.txt).
    /// PgBouncer default: not set (configure as needed)
    auth_file: Option<String>,

    /// Directory where the Unix domain socket is created.
    /// PgBouncer default: not set
    unix_socket_dir: Option<String>,

    /// Path to HBA configuration file when auth_type = hba.
    /// PgBouncer default: not set
    auth_hba_file: Option<String>,

    /// Path to ident map file.
    /// PgBouncer default: not set
    auth_ident_file: Option<String>,

    /// How long to keep released connections available before re-checking (seconds).
    /// PgBouncer default: 0
    server_check_delay: Option<i32>,

    /// If a server connection has been idle longer than this, close it (seconds).
    /// PgBouncer default: 3600
    server_idle_timeout: Option<i32>,

    /// Close an unused server connection that has been connected longer than this (seconds).
    /// PgBouncer default: 3600
    server_lifetime: Option<i32>,

    /// Timeout for establishing server connection and login (seconds).
    /// PgBouncer default: 15
    server_connect_timeout: Option<i32>,

    /// Wait time before retrying server login after failure (seconds).
    /// PgBouncer default: 15
    server_login_retry: Option<i32>,

    /// If a client connects but does not finish login within this time, disconnect (seconds).
    /// PgBouncer default: 15
    client_login_timeout: Option<i32>,

    /// Idle lifetime for automatically created (“*”) database pools (seconds).
    /// PgBouncer default: 60
    autodb_idle_timeout: Option<i32>,

    /// Maximum TTL to cache successful DNS lookups (seconds).
    /// PgBouncer default: 3600
    dns_max_ttl: Option<i32>,

    /// TTL to cache negative DNS results (NXDOMAIN) (seconds).
    /// PgBouncer default: 15
    dns_nxdomain_ttl: Option<i32>,

    /// Resolver configuration file path. If not set, use OS defaults.
    /// PgBouncer default: not set (use OS defaults)
    resolve_conf: Option<String>,

    /// Timeout for a single query execution (seconds). 0 disables.
    /// PgBouncer default: 0 (disabled)
    query_timeout: Option<i32>,

    /// Timeout for waiting on a server connection from pool (seconds).
    /// PgBouncer default: 120
    query_wait_timeout: Option<i32>,

    /// Timeout for forwarding CANCEL requests (seconds).
    /// PgBouncer default: 10
    cancel_wait_timeout: Option<i32>,

    /// Client idle timeout (seconds). 0 disables.
    /// PgBouncer default: 0 (disabled)
    client_idle_timeout: Option<i32>,

    /// Timeout for idle-in-transaction sessions (seconds). 0 disables.
    /// PgBouncer default: 0 (disabled)
    idle_transaction_timeout: Option<i32>,

    /// Timeout to wait for suspend to complete (seconds).
    /// PgBouncer default: 10
    suspend_timeout: Option<i32>,
}

impl PgBouncerSetting {
    pub(crate) fn new(
        listen_addr: &str,
        listen_port: u16,
        auth_type: AuthType,
        max_client_conn: u16,
        default_pool_size: u16,
        pool_mode: PoolMode,
        admin_users: Vec<&str>,
        stats_users: Vec<&str>,
        ignore_startup_parameters: Vec<&str>,
        logfile: Option<&str>,
        pidfile: Option<&str>,
        auth_file: Option<&str>,
        unix_socket_dir: Option<&str>,
        auth_hba_file: Option<&str>,
        auth_ident_file: Option<&str>,
        server_check_delay: Option<i32>,
        server_idle_timeout: Option<i32>,
        server_lifetime: Option<i32>,
        server_connect_timeout: Option<i32>,
        server_login_retry: Option<i32>,
        client_login_timeout: Option<i32>,
        autodb_idle_timeout: Option<i32>,
        dns_max_ttl: Option<i32>,
        dns_nxdomain_ttl: Option<i32>,
        resolve_conf: Option<&str>,
        query_timeout: Option<i32>,
        query_wait_timeout: Option<i32>,
        cancel_wait_timeout: Option<i32>,
        client_idle_timeout: Option<i32>,
        idle_transaction_timeout: Option<i32>,
        suspend_timeout: Option<i32>,
    ) -> Self {
        Self {
            listen_addr: listen_addr.to_string(),
            listen_port,
            auth_type,
            auth_file: auth_file.map(|a| a.to_string()),
            max_client_conn,
            default_pool_size,
            pool_mode,
            admin_users: admin_users.iter().map(|&user| user.to_string()).collect(),
            stats_users: stats_users.iter().map(|&user| user.to_string()).collect(),
            ignore_startup_parameters: ignore_startup_parameters.iter().map(|&param| param.to_string()).collect(),
            logfile: logfile.map(|file| file.to_string()),
            pidfile: pidfile.map(|file| file.to_string()),
            unix_socket_dir: unix_socket_dir.map(|dir| dir.to_string()),
            auth_hba_file: auth_hba_file.map(|file| file.to_string()),
            auth_ident_file: auth_ident_file.map(|file| file.to_string()),
            server_check_delay,
            server_idle_timeout,
            server_lifetime,
            server_connect_timeout,
            server_login_retry,
            client_login_timeout,
            autodb_idle_timeout,
            dns_max_ttl,
            dns_nxdomain_ttl,
            resolve_conf: resolve_conf.map(|file| file.to_string()),
            query_timeout,
            query_wait_timeout,
            cancel_wait_timeout,
            client_idle_timeout,
            idle_transaction_timeout,
            suspend_timeout,
        }
    }

    /// Set the listening address.
    ///
    /// # Parameters
    /// - addr: Desired listening address (IP or hostname).
    ///
    /// # Returns
    /// A cloned instance with the updated address.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_listen_addr("127.0.0.1");
    /// ```
    ///
    /// # Notes
    /// - Updates the `listen_addr` field.
    /// - This method does not parse "host:port"; set the port via [`set_listen_port`].
    pub fn set_listen_addr(&mut self, addr: &str) -> Self {
        self.listen_addr = addr.to_string();
        self.clone()
    }

    /// Set the listening port.
    ///
    /// # Parameters
    /// - port: Port number to listen on.
    ///
    /// # Returns
    /// A cloned instance with the updated port.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_listen_port(6432);
    /// ```
    pub fn set_listen_port(&mut self, port: u16) -> Self {
        self.listen_port = port;
        self.clone()
    }

    /// Set the authentication type.
    ///
    /// # Parameters
    /// - auth_type: Authentication method to use.
    ///
    /// # Returns
    /// A cloned instance with the updated authentication type.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::{AuthType, PgBouncerSetting};
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_auth_type(AuthType::ScramSha256);
    /// ```
    pub fn set_auth_type(&mut self, auth_type: AuthType) -> Self {
        self.auth_type = auth_type;
        self.clone()
    }

    /// Set the authentication file path.
    ///
    /// # Parameters
    /// - auth_file: Path to the authentication file.
    ///
    /// # Returns
    /// A cloned instance with the updated authentication file path.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_auth_file("/bitnami/pgbouncer-config/conf/userlist.txt");
    /// ```
    pub fn set_auth_file(&mut self, auth_file: &str) -> Self {
        self.auth_file = Some(auth_file.to_string());
        self.clone()
    }

    /// Set the maximum number of client connections.
    ///
    /// # Parameters
    /// - max_client_conn: Maximum number of allowed client connections.
    ///
    /// # Returns
    /// A cloned instance with the updated limit.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_max_client_conn(5000);
    /// ```
    pub fn set_max_client_conn(&mut self, max_client_conn: u16) -> Self {
        self.max_client_conn = max_client_conn;
        self.clone()
    }

    /// Set the default pool size.
    ///
    /// # Parameters
    /// - default_pool_size: Desired number of server connections per pool.
    ///
    /// # Returns
    /// A cloned instance with the updated pool size.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_default_pool_size(50);
    /// ```
    pub fn set_default_pool_size(&mut self, default_pool_size: u16) -> Self {
        self.default_pool_size = default_pool_size;
        self.clone()
    }

    /// Set the pool mode.
    ///
    /// # Parameters
    /// - pool_mode: New pooling mode to use.
    ///
    /// # Returns
    /// A cloned instance with the updated pool mode.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::{PgBouncerSetting, PoolMode};
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_pool_mode(PoolMode::Session);
    /// ```
    pub fn set_pool_mode(&mut self, pool_mode: PoolMode) -> Self {
        self.pool_mode = pool_mode;
        self.clone()
    }

    /// Add an admin user.
    ///
    /// # Parameters
    /// - user: Username to grant administrative privileges in PgBouncer.
    ///
    /// # Returns
    /// A cloned instance with the user added to `admin_users`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.add_admin_user("admin");
    /// ```
    pub fn add_admin_user(&mut self, user: &str) -> Self {
        self.admin_users.push(user.to_string());
        self.clone()
    }

    /// Add a statistics user.
    ///
    /// # Parameters
    /// - user: Username to grant permissions to view statistics only.
    ///
    /// # Returns
    /// A cloned instance with the user added to `stats_users`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.add_stats_user("stats_user");
    /// ```
    pub fn add_stats_user(&mut self, user: &str) -> Self {
        self.stats_users.push(user.to_string());
        self.clone()
    }

    /// Add an ignored startup parameter.
    ///
    /// Appends the given parameter to the `ignore_startup_parameters` list.
    ///
    /// # Parameters
    /// - param: Client startup parameter to ignore.
    ///
    /// # Returns
    /// A cloned instance with the updated list.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.add_ignore_startup_parameter("extra_float_digits");
    /// ```
    pub fn add_ignore_startup_parameter(&mut self, param: &str) -> Self {
    	self.ignore_startup_parameters.push(param.to_string());
    	self.clone()
    }

    /// Set the logfile path.
    ///
    /// # Parameters
    /// - logfile: Optional path to the logfile. `Some(path)` sets the logfile; `None` clears it.
    ///
    /// # Returns
    /// A cloned instance with the updated logfile path.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_logfile(Some("/path/to/logfile.log"));
    /// ```
    pub fn set_logfile(&mut self, logfile: Option<&str>) -> Self {
        self.logfile = logfile.map(|file| file.to_string());
        self.clone()
    }

    /// Set the PID file path.
    ///
    /// # Parameters
    /// - pidfile: Optional path to the PID file. `Some(path)` sets the PID file; `None` clears it.
    ///
    /// # Returns
    /// A cloned instance with the updated PID file path.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_pidfile(Some("/var/run/pgbouncer-config.pid"));
    /// config.set_pidfile(None);
    /// ```
    pub fn set_pidfile(&mut self, pidfile: Option<&str>) -> Self {
        self.pidfile = pidfile.map(|file| file.to_string());
        self.clone()
    }

    /// Set the Unix socket directory.
    ///
    /// # Parameters
    /// - unix_socket_dir: Optional directory path where the Unix socket is created.
    ///
    /// # Returns
    /// A cloned instance with the updated Unix socket directory.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_unix_socket_dir(Some("/tmp/socket_dir"));
    /// ```
    pub fn set_unix_socket_dir(&mut self, unix_socket_dir: Option<&str>) -> Self {
        self.unix_socket_dir = unix_socket_dir.map(|dir| dir.to_string());
        self.clone()
    }

    /// Set the HBA configuration file path.
    ///
    /// # Parameters
    /// - auth_hba_file: Optional path to the HBA configuration file.
    ///
    /// # Returns
    /// If successful, returns the updated configuration with the new HBA file path.
    ///
    /// # Errors
    /// Returns an error if `auth_type` is `AuthType::Hba` and `auth_hba_file` is `None`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::{PgBouncerSetting, AuthType};
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_auth_type(AuthType::Hba);
    /// config.set_auth_hba_file(Some("/etc/pgbouncer-config/pgb_hba.conf"))
    ///     .expect("hba file required for hba auth");
    /// ```
    pub fn set_auth_hba_file(&mut self, auth_hba_file: Option<&str>) -> crate::error::Result<Self> {
        if self.auth_type == AuthType::Hba && auth_hba_file.is_none() {
            return Err(PgBouncerError::PgBouncer(
                "auth_hba_file cannot be None when the auth_type is 'hba'".to_string()
            ));
        }

        self.auth_hba_file = auth_hba_file.map(|file| file.to_string());
        Ok(self.clone())
    }

    /// Set the ident map file path.
    ///
    /// # Parameters
    /// - auth_ident_file: Optional path to the ident map file.
    ///
    /// # Returns
    /// The updated configuration with the new ident map file path.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_auth_ident_file(Some("/etc/pgbouncer-config/pg_ident.map"));
    /// ```
    pub fn set_auth_ident_file(&mut self, auth_ident_file: Option<&str>) -> Self {
        self.auth_ident_file = auth_ident_file.map(|file| file.to_string());
        self.clone()
    }

    /// Set the server check delay.
    ///
    /// Defines how long to keep released server connections available before
    /// PgBouncer re-checks the connection health.
    ///
    /// # Parameters
    /// - secs: Optional delay in seconds. `Some(seconds)` sets the value; `None` clears it.
    ///
    /// # Returns
    /// A cloned instance with the updated `server_check_delay`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_server_check_delay(Some(5));
    /// config.set_server_check_delay(None);
    /// ```
    pub fn set_server_check_delay(&mut self, secs: Option<i32>) -> Self {
        self.server_check_delay = secs;
        self.clone()
    }

    /// Set the server idle timeout.
    ///
    /// If a server connection remains idle longer than this threshold,
    /// PgBouncer will close it.
    ///
    /// # Parameters
    /// - secs: Optional timeout in seconds. `Some(seconds)` sets the value; `None` clears it.
    ///
    /// # Returns
    /// A cloned instance with the updated `server_idle_timeout`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_server_idle_timeout(Some(3600));
    /// config.set_server_idle_timeout(None);
    /// ```
    pub fn set_server_idle_timeout(&mut self, secs: Option<i32>) -> Self {
        self.server_idle_timeout = secs;
        self.clone()
    }

    /// Set the server connection lifetime.
    ///
    /// Closes an unused server connection that has been connected longer than
    /// the specified time.
    ///
    /// # Parameters
    /// - secs: Optional lifetime in seconds. `Some(seconds)` sets the value; `None` clears it.
    ///
    /// # Returns
    /// A cloned instance with the updated `server_lifetime`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_server_lifetime(Some(3600));
    /// config.set_server_lifetime(None);
    /// ```
    pub fn set_server_lifetime(&mut self, secs: Option<i32>) -> Self {
        self.server_lifetime = secs;
        self.clone()
    }

    /// Set the server connect timeout.
    ///
    /// Limits how long PgBouncer waits when establishing a server connection
    /// and performing server-side login.
    ///
    /// # Parameters
    /// - secs: Optional timeout in seconds. `Some(seconds)` sets the value; `None` clears it.
    ///
    /// # Returns
    /// A cloned instance with the updated `server_connect_timeout`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_server_connect_timeout(Some(15));
    /// config.set_server_connect_timeout(None);
    /// ```
    pub fn set_server_connect_timeout(&mut self, secs: Option<i32>) -> Self {
        self.server_connect_timeout = secs;
        self.clone()
    }

    /// Set the server login retry delay.
    ///
    /// Controls the wait time before PgBouncer retries server login after a failure.
    ///
    /// # Parameters
    /// - secs: Optional delay in seconds. `Some(seconds)` sets the value; `None` clears it.
    ///
    /// # Returns
    /// A cloned instance with the updated `server_login_retry`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_server_login_retry(Some(15));
    /// config.set_server_login_retry(None);
    /// ```
    pub fn set_server_login_retry(&mut self, secs: Option<i32>) -> Self {
        self.server_login_retry = secs;
        self.clone()
    }

    /// Set the client login timeout.
    ///
    /// If a client connects but does not complete the login within this time,
    /// the client is disconnected.
    ///
    /// # Parameters
    /// - secs: Optional timeout in seconds. `Some(seconds)` sets the value; `None` clears it.
    ///
    /// # Returns
    /// A cloned instance with the updated `client_login_timeout`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_client_login_timeout(Some(15));
    /// config.set_client_login_timeout(None);
    /// ```
    pub fn set_client_login_timeout(&mut self, secs: Option<i32>) -> Self {
        self.client_login_timeout = secs;
        self.clone()
    }

    /// Set the autodb idle timeout.
    ///
    /// Controls the idle lifetime for automatically created (“*”) database pools.
    ///
    /// # Parameters
    /// - secs: Optional timeout in seconds. `Some(seconds)` sets the value; `None` clears it.
    ///
    /// # Returns
    /// A cloned instance with the updated `autodb_idle_timeout`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_autodb_idle_timeout(Some(60));
    /// config.set_autodb_idle_timeout(None);
    /// ```
    pub fn set_autodb_idle_timeout(&mut self, secs: Option<i32>) -> Self {
        self.autodb_idle_timeout = secs;
        self.clone()
    }

    /// Set the maximum DNS positive cache TTL.
    ///
    /// Defines the maximum time to cache successful DNS lookups.
    ///
    /// # Parameters
    /// - secs: Optional TTL in seconds. `Some(seconds)` sets the value; `None` clears it.
    ///
    /// # Returns
    /// A cloned instance with the updated `dns_max_ttl`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_dns_max_ttl(Some(3600));
    /// config.set_dns_max_ttl(None);
    /// ```
    pub fn set_dns_max_ttl(&mut self, secs: Option<i32>) -> Self {
        self.dns_max_ttl = secs;
        self.clone()
    }

    /// Set the DNS negative cache TTL (NXDOMAIN).
    ///
    /// Defines how long negative DNS results (NXDOMAIN) are cached.
    ///
    /// # Parameters
    /// - secs: Optional TTL in seconds. `Some(seconds)` sets the value; `None` clears it.
    ///
    /// # Returns
    /// A cloned instance with the updated `dns_nxdomain_ttl`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_dns_nxdomain_ttl(Some(15));
    /// config.set_dns_nxdomain_ttl(None);
    /// ```
    pub fn set_dns_nxdomain_ttl(&mut self, secs: Option<i32>) -> Self {
        self.dns_nxdomain_ttl = secs;
        self.clone()
    }

    /// Set the resolver configuration file path.
    ///
    /// If not set, system defaults are used.
    ///
    /// # Parameters
    /// - path: Optional file path to the resolver configuration. `Some(path)` sets the file; `None` clears it.
    ///
    /// # Returns
    /// A cloned instance with the updated `resolve_conf`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_resolve_conf(Some("/etc/resolv.conf"));
    /// config.set_resolve_conf(None);
    /// ```
    pub fn set_resolve_conf(&mut self, path: Option<&str>) -> Self {
        self.resolve_conf = path.map(|p| p.to_string());
        self.clone()
    }

    /// Set the query execution timeout.
    ///
    /// Limits how long a single query is allowed to run. A value of `0` disables
    /// the timeout if configured in PgBouncer; setting `None` clears this option.
    ///
    /// # Parameters
    /// - secs: Optional timeout in seconds. `Some(seconds)` sets the value; `None` clears it.
    ///
    /// # Returns
    /// A cloned instance with the updated `query_timeout`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_query_timeout(Some(30));
    /// config.set_query_timeout(None);
    /// ```
    pub fn set_query_timeout(&mut self, secs: Option<i32>) -> Self {
        self.query_timeout = secs;
        self.clone()
    }

    /// Set the query wait timeout.
    ///
    /// Controls how long a client waits for a server connection from the pool.
    ///
    /// # Parameters
    /// - secs: Optional timeout in seconds. `Some(seconds)` sets the value; `None` clears it.
    ///
    /// # Returns
    /// A cloned instance with the updated `query_wait_timeout`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_query_wait_timeout(Some(120));
    /// config.set_query_wait_timeout(None);
    /// ```
    pub fn set_query_wait_timeout(&mut self, secs: Option<i32>) -> Self {
        self.query_wait_timeout = secs;
        self.clone()
    }

    /// Set the cancel request wait timeout.
    ///
    /// Controls the timeout for forwarding CANCEL requests to the server.
    ///
    /// # Parameters
    /// - secs: Optional timeout in seconds. `Some(seconds)` sets the value; `None` clears it.
    ///
    /// # Returns
    /// A cloned instance with the updated `cancel_wait_timeout`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_cancel_wait_timeout(Some(10));
    /// config.set_cancel_wait_timeout(None);
    /// ```
    pub fn set_cancel_wait_timeout(&mut self, secs: Option<i32>) -> Self {
        self.cancel_wait_timeout = secs;
        self.clone()
    }

    /// Set the client idle timeout.
    ///
    /// Disconnects a client if it remains idle longer than the specified time.
    /// A value of `0` disables the timeout; setting `None` clears this option.
    ///
    /// # Parameters
    /// - secs: Optional timeout in seconds. `Some(seconds)` sets the value; `None` clears it.
    ///
    /// # Returns
    /// A cloned instance with the updated `client_idle_timeout`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_client_idle_timeout(Some(0));
    /// config.set_client_idle_timeout(Some(600));
    /// config.set_client_idle_timeout(None);
    /// ```
    pub fn set_client_idle_timeout(&mut self, secs: Option<i32>) -> Self {
        self.client_idle_timeout = secs;
        self.clone()
    }

    /// Set the idle-in-transaction timeout.
    ///
    /// Terminates sessions that remain idle while in a transaction longer than
    /// the specified time. A value of `0` disables the timeout; setting `None` clears it.
    ///
    /// # Parameters
    /// - secs: Optional timeout in seconds. `Some(seconds)` sets the value; `None` clears it.
    ///
    /// # Returns
    /// A cloned instance with the updated `idle_transaction_timeout`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_idle_transaction_timeout(Some(0));
    /// config.set_idle_transaction_timeout(Some(300));
    /// config.set_idle_transaction_timeout(None);
    /// ```
    pub fn set_idle_transaction_timeout(&mut self, secs: Option<i32>) -> Self {
        self.idle_transaction_timeout = secs;
        self.clone()
    }

    /// Set the suspend timeout.
    ///
    /// Controls how long PgBouncer waits for a suspend operation to complete.
    ///
    /// # Parameters
    /// - secs: Optional timeout in seconds. `Some(seconds)` sets the value; `None` clears it.
    ///
    /// # Returns
    /// A cloned instance with the updated `suspend_timeout`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    ///
    /// let mut config = PgBouncerSetting::default();
    /// config.set_suspend_timeout(Some(10));
    /// config.set_suspend_timeout(None);
    /// ```
    pub fn set_suspend_timeout(&mut self, secs: Option<i32>) -> Self {
        self.suspend_timeout = secs;
        self.clone()
    }
}

impl Default for PgBouncerSetting {
    fn default() -> Self {
        Self::new(
            "127.0.0.1",
            6432,
            AuthType::default(),
            2000,
            100,
            PoolMode::default(),
            vec![],
            vec![],
            vec![],
            None, 
            None,
            Some("/etc/pgbouncer-config/userlist.txt"),
            None, 
            None, 
            None, 
            None, 
            None, 
            None, 
            None, 
            None, 
            None, 
            None, 
            None,
            None, 
            None, 
            None, 
            None, 
            None, 
            None, 
            None, 
            None,
        )
    }
}

#[typetag::serde]
impl Expression for PgBouncerSetting {
    fn expr(&self) -> String {
        let mut expr = "[pgbouncer]\n".to_string();
        expr.push_str(&format!("listen_addr = {}\n", self.listen_addr));
        expr.push_str(&format!("listen_port = {}\n", self.listen_port));
        expr.push_str(&format!("auth_type = {}\n", self.auth_type));
        expr.push_str(&format!("max_client_conn = {}\n", self.max_client_conn));
        expr.push_str(&format!("default_pool_size = {}\n", self.default_pool_size));
        expr.push_str(&format!("pool_mode = {}\n", self.pool_mode));

        if self.admin_users.len() > 0 {
            expr.push_str(&format!("admin_users = {}\n", self.admin_users.join(",")));
        }
        if self.stats_users.len() > 0 {
            expr.push_str(&format!("stats_users = {}\n", self.stats_users.join(",")));
        }
        if self.ignore_startup_parameters.len() > 0 {
            expr.push_str(&format!("ignore_startup_parameters = {}\n", self.ignore_startup_parameters.join(",")));
        }

        if let Some(logfile) = &self.logfile {
            expr.push_str(&format!("logfile = {}\n", logfile));
        }
        if let Some(pidfile) = &self.pidfile {
            expr.push_str(&format!("pidfile = {}\n", pidfile));
        }
        if let Some(auth_file) = &self.auth_file {
            expr.push_str(&format!("auth_file = {}\n", auth_file));
        }
        if let Some(unix_socket_dir) = &self.unix_socket_dir {
            expr.push_str(&format!("unix_socket_dir = {}\n", unix_socket_dir));
        }
        if let Some(auth_hba_file) = &self.auth_hba_file {
            expr.push_str(&format!("auth_hba_file = {}\n", auth_hba_file));
        }
        if let Some(auth_ident_file) = &self.auth_ident_file {
            expr.push_str(&format!("auth_ident_file = {}\n", auth_ident_file));
        }

        expr
    }

    fn config_section_name(&self) -> &'static str {
        "PgBouncerInfo"
    }
}

#[cfg(feature = "io")]
impl ParserIniFromStr for PgBouncerSetting {
    type Error = PgBouncerError;

    fn parse_from_str(value: &str) -> Result<Self, Self::Error> {
        let mut pgbouncer_setting = HashMap::new();

        for line in value.lines() {
            let (k, v) = parse_key_value(line)?;
            pgbouncer_setting.insert(k, v);
        }

        let listen_addr = pgbouncer_setting.get("listen_addr")
            .ok_or(
                PgBouncerError::PgBouncer("listen_addr is required in [pgbouncer] section".to_string())
            )?
            .to_string();
        let listen_port: u16 = pgbouncer_setting.get("listen_port")
            .ok_or(
                PgBouncerError::PgBouncer("listen_port is required in [pgbouncer] section".to_string())
            )?
            .parse()
            .map_err(|_| PgBouncerError::PgBouncer("listen_port must be a number".to_string()))?;
        let auth_type_str = pgbouncer_setting.get("auth_type")
            .ok_or(
                PgBouncerError::PgBouncer("auth_type is required in [pgbouncer] section".to_string())
            )?.to_string();
        let auth_type = AuthType::try_from(auth_type_str)?;

        let max_client_conn: u16 = pgbouncer_setting.get("max_client_conn")
            .ok_or(
                PgBouncerError::PgBouncer("max_client_conn is required in [pgbouncer] section".to_string())
            )?
            .parse()
            .map_err(|_| PgBouncerError::PgBouncer("max_client_conn must be a number".to_string()))?;

        let default_pool_size: u16 = pgbouncer_setting.get("default_pool_size")
            .ok_or(
                PgBouncerError::PgBouncer("default_pool_size is required in [pgbouncer] section".to_string())
            )?
            .parse()
            .map_err(|_| PgBouncerError::PgBouncer("default_pool_size must be a number".to_string()))?;

        let pool_mode = match pgbouncer_setting.get("pool_mode")
            .ok_or(PgBouncerError::PgBouncer("pool_mode is required in [pgbouncer] section".to_string()))? {
            s if s.eq_ignore_ascii_case("session") => PoolMode::Session,
            s if s.eq_ignore_ascii_case("transaction") => PoolMode::Transaction,
            s if s.eq_ignore_ascii_case("statement") => PoolMode::Statement,
            other => return Err(PgBouncerError::PgBouncer(format!("Invalid pool_mode: {}", other))),
        };

        let admin_users = pgbouncer_setting.get("admin_users")
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        let stats_users = pgbouncer_setting.get("stats_users")
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        let ignore_startup_parameters = pgbouncer_setting.get("ignore_startup_parameters")
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        let logfile = pgbouncer_setting.get("logfile").map(|s| s.to_string());
        let pidfile = pgbouncer_setting.get("pidfile").map(|s| s.to_string());
        let auth_file = pgbouncer_setting.get("auth_file").map(|s| s.to_string());
        let unix_socket_dir = pgbouncer_setting.get("unix_socket_dir").map(|s| s.to_string());
        let auth_hba_file = pgbouncer_setting.get("auth_hba_file").map(|s| s.to_string());
        let auth_ident_file = pgbouncer_setting.get("auth_ident_file").map(|s| s.to_string());

        let server_check_delay = pgbouncer_setting.get("server_check_delay")
            .map(|v| v.parse::<i32>())
            .transpose()
            .map_err(|_| PgBouncerError::PgBouncer("server_check_delay must be a number".to_string()))?;

        let server_idle_timeout = pgbouncer_setting.get("server_idle_timeout")
            .map(|v| v.parse::<i32>())
            .transpose()
            .map_err(|_| PgBouncerError::PgBouncer("server_idle_timeout must be a number".to_string()))?;

        let server_lifetime = pgbouncer_setting.get("server_lifetime")
            .map(|v| v.parse::<i32>())
            .transpose()
            .map_err(|_| PgBouncerError::PgBouncer("server_lifetime must be a number".to_string()))?;

        let server_connect_timeout = pgbouncer_setting.get("server_connect_timeout")
            .map(|v| v.parse::<i32>())
            .transpose()
            .map_err(|_| PgBouncerError::PgBouncer("server_connect_timeout must be a number".to_string()))?;

        let server_login_retry = pgbouncer_setting.get("server_login_retry")
            .map(|v| v.parse::<i32>())
            .transpose()
            .map_err(|_| PgBouncerError::PgBouncer("server_login_retry must be a number".to_string()))?;

        let client_login_timeout = pgbouncer_setting.get("client_login_timeout")
            .map(|v| v.parse::<i32>())
            .transpose()
            .map_err(|_| PgBouncerError::PgBouncer("client_login_timeout must be a number".to_string()))?;

        let autodb_idle_timeout = pgbouncer_setting.get("autodb_idle_timeout")
            .map(|v| v.parse::<i32>())
            .transpose()
            .map_err(|_| PgBouncerError::PgBouncer("autodb_idle_timeout must be a number".to_string()))?;

        let dns_max_ttl = pgbouncer_setting.get("dns_max_ttl")
            .map(|v| v.parse::<i32>())
            .transpose()
            .map_err(|_| PgBouncerError::PgBouncer("dns_max_ttl must be a number".to_string()))?;

        let dns_nxdomain_ttl = pgbouncer_setting.get("dns_nxdomain_ttl")
            .map(|v| v.parse::<i32>())
            .transpose()
            .map_err(|_| PgBouncerError::PgBouncer("dns_nxdomain_ttl must be a number".to_string()))?;

        let resolve_conf = pgbouncer_setting.get("resolve_conf").map(|s| s.to_string());

        let query_timeout = pgbouncer_setting.get("query_timeout")
            .map(|v| v.parse::<i32>())
            .transpose()
            .map_err(|_| PgBouncerError::PgBouncer("query_timeout must be a number".to_string()))?;

        let query_wait_timeout = pgbouncer_setting.get("query_wait_timeout")
            .map(|v| v.parse::<i32>())
            .transpose()
            .map_err(|_| PgBouncerError::PgBouncer("query_wait_timeout must be a number".to_string()))?;

        let cancel_wait_timeout = pgbouncer_setting.get("cancel_wait_timeout")
            .map(|v| v.parse::<i32>())
            .transpose()
            .map_err(|_| PgBouncerError::PgBouncer("cancel_wait_timeout must be a number".to_string()))?;

        let client_idle_timeout = pgbouncer_setting.get("client_idle_timeout")
            .map(|v| v.parse::<i32>())
            .transpose()
            .map_err(|_| PgBouncerError::PgBouncer("client_idle_timeout must be a number".to_string()))?;

        let idle_transaction_timeout = pgbouncer_setting.get("idle_transaction_timeout")
            .map(|v| v.parse::<i32>())
            .transpose()
            .map_err(|_| PgBouncerError::PgBouncer("idle_transaction_timeout must be a number".to_string()))?;

        let suspend_timeout = pgbouncer_setting.get("suspend_timeout")
            .map(|v| v.parse::<i32>())
            .transpose()
            .map_err(|_| PgBouncerError::PgBouncer("suspend_timeout must be a number".to_string()))?;

        Ok(Self {
            listen_addr,
            listen_port,
            auth_type,
            max_client_conn,
            default_pool_size,
            pool_mode,
            admin_users,
            stats_users,
            ignore_startup_parameters,
            logfile,
            pidfile,
            auth_file,
            unix_socket_dir,
            auth_hba_file,
            auth_ident_file,
            server_check_delay,
            server_idle_timeout,
            server_lifetime,
            server_connect_timeout,
            server_login_retry,
            client_login_timeout,
            autodb_idle_timeout,
            dns_max_ttl,
            dns_nxdomain_ttl,
            resolve_conf,
            query_timeout,
            query_wait_timeout,
            cancel_wait_timeout,
            client_idle_timeout,
            idle_transaction_timeout,
            suspend_timeout,
        })
    }
}

#[cfg(feature = "diff")]
#[typetag::serde]
impl Diffable for PgBouncerSetting {}

/// Authentication type used by PgBouncer.
///
/// Controls how clients are authenticated. See the official PgBouncer
/// documentation for detailed behavior of each method.
#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AuthType {
    #[default]
    #[serde(rename = "md5")]
    Md5,

    #[serde(rename = "scram-sha-256")]
    ScramSha256,
    #[serde(rename = "cert")]
    Cert,
    #[serde(rename = "plain")]
    Plain,
    #[serde(rename = "trust")]
    Trust,
    #[serde(rename = "any")]
    Any,
    #[serde(rename = "hba")]
    Hba,
    #[serde(rename = "pam")]
    Pam,
}

impl TryFrom<&str> for AuthType {
    type Error = PgBouncerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let auth_type = match value.to_lowercase().as_str() {
            "md5" => AuthType::Md5,
            "sha256" | "scram-sha-256" | "scram_sha_256" | "scram-sha256" | "scram_sha256" | "scramsha256" => AuthType::ScramSha256,
            "cert" => AuthType::Cert,
            "plain" => AuthType::Plain,
            "trust" => AuthType::Trust,
            "any" => AuthType::Any,
            "hba" => AuthType::Hba,
            "pam" => AuthType::Pam,
            _ => {
                let error_msg = format!("Unsupported auth_type: {}", value);
                return Err(PgBouncerError::PgBouncer(error_msg));
            }
        };

        Ok(auth_type)
    }
}

impl TryFrom<String> for AuthType {
    type Error = PgBouncerError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl Display for AuthType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthType::Md5 => write!(f, "md5"),
            AuthType::ScramSha256 => write!(f, "scram-sha-256"),
            AuthType::Cert => write!(f, "cert"),
            AuthType::Plain => write!(f, "plain"),
            AuthType::Trust => write!(f, "trust"),
            AuthType::Any => write!(f, "any"),
            AuthType::Hba => write!(f, "hba"),
            AuthType::Pam => write!(f, "pam"),
        }
    }
}

/// Connection pooling mode.
///
/// Determines how server connections are assigned to clients:
/// - Session: one server per client session (default).
/// - Transaction: server assigned per transaction.
/// - Statement: server assigned per statement.
#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PoolMode {
    #[default]
    Session,
    Transaction,
    Statement,
}

impl Display for PoolMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolMode::Session => write!(f, "session"),
            PoolMode::Transaction => write!(f, "transaction"),
            PoolMode::Statement => write!(f, "statement"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expr_includes_header_and_basic_fields_after_setters() {
        let mut s = PgBouncerSetting::default();
        s = s
            .set_listen_addr("0.0.0.0")
            .set_listen_port(6432)
            .set_auth_type(AuthType::Md5)
            .set_max_client_conn(200)
            .set_default_pool_size(50)
            .set_pool_mode(PoolMode::Session)
            .add_admin_user("admin1")
            .add_stats_user("stats1")
            .add_ignore_startup_parameter("search_path")
            .set_logfile(Some("/var/log/pgbouncer.log"))
            .set_pidfile(Some("/var/run/pgbouncer.pid"))
            .set_auth_file("/etc/pgbouncer/userlist.txt")
            .set_unix_socket_dir(Some("/var/run"))
            .set_auth_ident_file(Some("/etc/pgbouncer/ident.map"));

        let text = s.expr();
        assert!(text.starts_with("[pgbouncer]\n"));
        assert!(text.contains("listen_addr = 0.0.0.0"));
        assert!(text.contains("listen_port = 6432"));
        assert!(text.contains("auth_type = md5"));
        assert!(text.contains("max_client_conn = 200"));
        assert!(text.contains("default_pool_size = 50"));
        assert!(text.contains("pool_mode = session"));
        assert!(text.contains("admin_users = admin1"));
        assert!(text.contains("stats_users = stats1"));
        assert!(text.contains("ignore_startup_parameters = search_path"));
        assert!(text.contains("logfile = /var/log/pgbouncer.log"));
        assert!(text.contains("pidfile = /var/run/pgbouncer.pid"));
        assert!(text.contains("auth_file = /etc/pgbouncer/userlist.txt"));
        assert!(text.contains("unix_socket_dir = /var/run"));
        assert!(text.contains("auth_ident_file = /etc/pgbouncer/ident.map"));
    }

    #[test]
    fn auth_type_try_from_and_display() {
        // Lower-case and dashes should be accepted per TryFrom
        assert!(matches!(AuthType::try_from("md5"), Ok(AuthType::Md5)));
        assert!(matches!(AuthType::try_from("scram-sha-256"), Ok(AuthType::ScramSha256)));
        assert!(matches!(AuthType::try_from("cert"), Ok(AuthType::Cert)));
        assert!(matches!(AuthType::try_from("plain"), Ok(AuthType::Plain)));
        assert!(matches!(AuthType::try_from("trust"), Ok(AuthType::Trust)));
        assert!(matches!(AuthType::try_from("any"), Ok(AuthType::Any)));
        assert!(matches!(AuthType::try_from("hba"), Ok(AuthType::Hba)));
        assert!(matches!(AuthType::try_from("pam"), Ok(AuthType::Pam)));

        // Display should serialize back to canonical names
        assert_eq!(format!("{}", AuthType::Md5), "md5");
        assert_eq!(format!("{}", AuthType::ScramSha256), "scram-sha-256");
        assert_eq!(format!("{}", AuthType::Cert), "cert");
        assert_eq!(format!("{}", AuthType::Plain), "plain");
        assert_eq!(format!("{}", AuthType::Trust), "trust");
        assert_eq!(format!("{}", AuthType::Any), "any");
        assert_eq!(format!("{}", AuthType::Hba), "hba");
        assert_eq!(format!("{}", AuthType::Pam), "pam");
    }

    #[test]
    fn pool_mode_display_values() {
        assert_eq!(format!("{}", PoolMode::Session), "session");
        assert_eq!(format!("{}", PoolMode::Transaction), "transaction");
        assert_eq!(format!("{}", PoolMode::Statement), "statement");
    }
}
