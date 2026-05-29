use anyhow::Result;
use std::path::PathBuf;

/// MySQL connection parameters.
#[derive(Debug, Clone)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

/// Server configuration loaded from environment variables or .env file
#[derive(Debug, Clone)]
pub struct Config {
    pub db: DbConfig,

    /// AES-256 encryption key (64 hex characters = 32 bytes)
    pub aes_key_hex: String,

    /// Path to RSA private key PEM file
    pub rsa_private_key_path: PathBuf,

    /// Server bind address (e.g., "0.0.0.0:5505")
    pub bind_address: String,

    /// Starting cera balance granted to newly registered accounts.
    pub initial_cera: u32,

    /// Starting cera_point balance granted to newly registered accounts.
    pub initial_cera_point: u32,

    /// Path to the TLS certificate file in PEM format, may include intermediate CA certificates.
    pub tls_cert_path: Option<PathBuf>,

    /// Path to the TLS private key file in PEM format.
    pub tls_key_path: Option<PathBuf>,

    /// Bind address for the HTTPS listener (e.g., "0.0.0.0:5504").
    pub tls_bind_address: String,

    /// Disables the HTTP listener when TLS is active.
    pub tls_only: bool,

    /// Game server IP, reported to clients via GET /api/v1/game-server-ip.
    pub game_server_ip: Option<String>,

    /// Whether the server accepts new user registration.
    pub registration_open: bool,
}

impl Config {
    /// Load configuration from environment variables (or .env file).
    ///
    /// Database variables:
    ///   DB_HOST      — MySQL host          (default: 127.0.0.1)
    ///   DB_PORT      — MySQL port          (default: 3306)
    ///   DB_USER      — MySQL username      (default: game)
    ///   DB_PASSWORD  — MySQL password, plain text, no encoding needed
    ///   DB_NAME      — database name       (default: d_taiwan)
    ///
    /// Other variables:
    ///   AES_KEY              — 64 hex chars (32 bytes)
    ///   RSA_PRIVATE_KEY_PATH — path to PEM file (default: ./keys/private_key.pem)
    ///   BIND_ADDRESS         — listen address  (default: 0.0.0.0:5505)
    ///   INITIAL_CERA         — starting cera balance on registration (default: 1000)
    ///   INITIAL_CERA_POINT   — starting cera_point balance          (default: 0)
    ///   TLS_CERT_PATH        — PEM certificate file, may include the full chain
    ///   TLS_KEY_PATH         — PEM private key file
    ///   TLS_BIND_ADDRESS     — HTTPS listen address (default: 0.0.0.0:5504)
    ///   TLS_ONLY             — disable HTTP when TLS is active (default: false)
    ///   REGISTRATION_OPEN    — accept new user registration (default: true)
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let db = DbConfig {
            host: std::env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("DB_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3306),
            user: std::env::var("DB_USER").unwrap_or_else(|_| "game".to_string()),
            password: std::env::var("DB_PASSWORD").map_err(|_| {
                anyhow::anyhow!("DB_PASSWORD environment variable is required but not set")
            })?,
            database: std::env::var("DB_NAME").unwrap_or_else(|_| "d_taiwan".to_string()),
        };

        tracing::info!(
            "Database: {}@{}:{}/{}",
            db.user,
            db.host,
            db.port,
            db.database
        );

        let aes_key_hex = std::env::var("AES_KEY").map_err(|_| {
            anyhow::anyhow!(
                "AES_KEY environment variable is required but not set. \
                 Generate a key with: openssl rand -hex 32"
            )
        })?;
        if aes_key_hex.len() != 64 || !aes_key_hex.chars().all(|c| c.is_ascii_hexdigit()) {
            anyhow::bail!(
                "AES_KEY must be exactly 64 hexadecimal characters (got {}). \
                 Generate a valid key with: openssl rand -hex 32",
                aes_key_hex.len()
            );
        }

        let rsa_private_key_path = std::env::var("RSA_PRIVATE_KEY_PATH")
            .unwrap_or_else(|_| "./keys/private_key.pem".to_string())
            .into();

        let bind_address =
            std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:5505".to_string());

        let initial_cera = std::env::var("INITIAL_CERA")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1000u32);

        let initial_cera_point = std::env::var("INITIAL_CERA_POINT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0u32);

        let tls_cert_path = std::env::var("TLS_CERT_PATH").ok().map(PathBuf::from);
        let tls_key_path = std::env::var("TLS_KEY_PATH").ok().map(PathBuf::from);
        let tls_bind_address =
            std::env::var("TLS_BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:5504".to_string());
        let tls_only = std::env::var("TLS_ONLY")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(false);

        let game_server_ip = std::env::var("GAME_SERVER_IP")
            .ok()
            .map(|s| {
                let trimmed = s.trim().to_string();
                trimmed.parse::<std::net::IpAddr>().map_err(|_| {
                    anyhow::anyhow!(
                        "GAME_SERVER_IP must be a valid IP address, got: {:?}",
                        trimmed
                    )
                })?;
                Ok::<String, anyhow::Error>(trimmed)
            })
            .transpose()?;

        let registration_open = std::env::var("REGISTRATION_OPEN")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(true);

        Ok(Self {
            db,
            aes_key_hex,
            rsa_private_key_path,
            bind_address,
            initial_cera,
            initial_cera_point,
            tls_cert_path,
            tls_key_path,
            tls_bind_address,
            tls_only,
            game_server_ip,
            registration_open,
        })
    }
}
