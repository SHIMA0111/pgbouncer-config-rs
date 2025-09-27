use std::net::SocketAddr;
use std::sync::Arc;
use russh::client;
use russh::keys::{decode_secret_key, load_secret_key, HashAlg, PrivateKeyWithHashAlg, PublicKey};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use crate::error::PgBouncerError;
use crate::pgbouncer_config::databases_setting::{SSHAuth, SSHTunnelBuilder};

struct ClientHandler;

impl client::Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(&mut self, _server_public_key: &PublicKey) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

pub struct SSHTunnel {
    bastion_host: String,
    bastion_port: u16,
    bastion_user: String,
    bastion_auth: SSHAuth,
    local_port: u16,
    pg_host: String,
    pg_port: u16,
}

pub struct SSHTunnelHandler {
    shutdown_tx: watch::Sender<()>,
    local_addr: SocketAddr,
}

impl SSHTunnelHandler {
    pub async fn shutdown(self) {
        drop(self.shutdown_tx);
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
}

impl SSHTunnel {
    pub fn new(
        bastion_host: &str,
        bastion_port: u16,
        bastion_user: &str,
        bastion_auth: SSHAuth,
        local_port: u16,
        pg_host: &str,
        pg_port: u16,
    ) -> Self {
        Self {
            bastion_host: bastion_host.to_string(),
            bastion_port,
            bastion_user: bastion_user.to_string(),
            bastion_auth,
            local_port,
            pg_host: pg_host.to_string(),
            pg_port,
        }
    }

    pub async fn run(&self) -> crate::error::Result<SSHTunnelHandler> {
        let (shutdown_tx, mut shutdown_rx) = watch::channel(());

        let config = Arc::new(client::Config::default());
        let client_handler = ClientHandler;
        let mut session = client::connect(
            config, (self.bastion_host(), self.bastion_port), client_handler).await?;

        let auth_success = match self.bastion_auth() {
            SSHAuth::Password(password) => {
                session.authenticate_password(self.bastion_user(), password).await?
            },
            SSHAuth::SSHKey {
                key, pass_phrase
            } => {
                let key_pair = decode_secret_key(&key, pass_phrase.as_deref())?;
                session.authenticate_publickey(
                    self.bastion_user(),
                    PrivateKeyWithHashAlg::new(
                        Arc::new(key_pair),
                        None
                    )
                ).await?
            },
            SSHAuth::LocalSSHKeyFile {
                path, pass_phrase
            } => {
                let key_pair = load_secret_key(path.as_path(), pass_phrase.as_deref())?;
                session.authenticate_publickey(
                    self.bastion_user(),
                    PrivateKeyWithHashAlg::new(
                        Arc::new(key_pair),
                        Some(HashAlg::Sha256)
                    )
                ).await?
            }
        };

        if !auth_success.success() {
            return Err(PgBouncerError::Connection(format!("Authentication failed for user {}", self.bastion_user())));
        }

        let listener = TcpListener::bind(("127.0.0.1", self.local_port)).await?;
        let local_addr = listener.local_addr()?;
        let session_arc = Arc::new(session);

        let session_arc_clone = session_arc.clone();
        let pg_host = self.pg_host.clone();
        let pg_port = self.pg_port;
        tokio::spawn(async move {
            loop {
                let session_handle = session_arc_clone.clone();
                let pg_host = pg_host.clone();
                tokio::select! {
                    accepted = listener.accept() => {
                        match accepted {
                            Ok((socket, addr)) => {
                                tokio::spawn(async move {
                                    if let Err(e) = Self::handle_connection(
                                        session_handle,
                                        socket,
                                        addr,
                                        &pg_host,
                                        pg_port,
                                    ).await {
                                        log::error!("Error handling connection: {}", e);
                                    }
                                });
                            },
                            Err(e) => {
                                log::error!("Error accepting connection: {}", e);
                            }
                        }
                    },
                    _ = shutdown_rx.changed() => {
                        log::info!("Shutting down");
                        break;
                    }
                }
            }

            if let Err(e) = session_arc.disconnect(russh::Disconnect::ByApplication, "Shutdown", "en").await {
                return Err(PgBouncerError::Connection(format!("Disconnect error: {}", e)));
            }

            Ok(())
        });


        Ok(SSHTunnelHandler { shutdown_tx, local_addr })
    }

    fn bastion_host(&self) -> &str {
        &self.bastion_host
    }

    fn bastion_user(&self) -> &str {
        &self.bastion_user
    }

    fn bastion_auth(&self) -> &SSHAuth {
        &self.bastion_auth
    }

    async fn handle_connection(
        session_handle: Arc<client::Handle<ClientHandler>>,
        mut local_socket: TcpStream,
        client_addr: SocketAddr,
        pg_host: &str,
        pg_port: u16,
    ) -> crate::error::Result<()> {
        let channel = match session_handle.channel_open_direct_tcpip(
            pg_host,
            pg_port as u32,
            "127.0.0.1",
            local_socket.local_addr()?.port() as u32,
        ).await {
            Ok(channel) => channel,
            Err(e) => {
                return Err(PgBouncerError::Connection(format!("Failed to open TCP: {}", e)));
            }
        };

        let mut channel_stream = channel.into_stream();

        match tokio::io::copy_bidirectional(&mut local_socket, &mut channel_stream).await {
            Ok((up, down)) => {
                log::debug!(
                "Connection from {} closed. Bytes uploaded: {}, downloaded: {}",
                client_addr,
                up,
                down,
            );
            },
            Err(e) => {
                return Err(PgBouncerError::Connection(format!("Error reading bidirectional: {}", e)));
            }
        }

        Ok(())
    }
}

impl From<SSHTunnelBuilder> for SSHTunnel {
    fn from(value: SSHTunnelBuilder) -> Self {
        let bastion_port = value.port.unwrap_or(22);
        // If the port is 0 in TcpListener means auto get port.
        let local_port = value.local_port.unwrap_or(0);
        let pg_port = value.remote_port.unwrap_or(5432);

        Self {
            bastion_host: value.host,
            bastion_port,
            bastion_user: value.user,
            bastion_auth: value.auth,
            local_port,
            pg_host: value.remote_host,
            pg_port,
        }
    }
}
