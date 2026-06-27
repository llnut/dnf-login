// dnf-login: a launcher for Dungeon & Fighter written in Rust.
// Copyright (C) 2026 llnut
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use axum_server::tls_rustls::RustlsConfig;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower_http::trace::TraceLayer;

mod config;
mod db;
mod handlers;
mod services;

use config::Config;
use handlers::{AppState, encrypted_request, game_server_ip, health};
use services::AuthService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting DNF Login Server...");

    let config = Config::from_env()?;
    tracing::info!("Configuration loaded");

    let pool = db::create_pool(&config.db).await?;
    db::test_connection(&pool).await?;

    let rsa_pem = tokio::fs::read_to_string(&config.rsa_private_key_path).await?;
    let token_generator = dnf_shared::crypto::TokenGenerator::from_pem(&rsa_pem)?;
    tracing::info!(
        "RSA private key loaded (key size: {} bits)",
        token_generator.key_size()
    );

    let aes_cipher = dnf_shared::crypto::AesGcmCipher::from_hex_key(&config.aes_key_hex)?;
    tracing::info!("AES-256-GCM cipher initialized");

    let auth_service = AuthService::new(
        pool.clone(),
        token_generator,
        config.initial_cera,
        config.initial_cera_point,
    );

    let state = AppState {
        auth_service: Arc::new(auth_service),
        aes_cipher: Arc::new(aes_cipher),
        rate_limiter: Arc::new(Mutex::new(HashMap::new())),
        game_server_ip: config.game_server_ip.clone(),
        registration_open: config.registration_open,
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/game-server-ip", get(game_server_ip))
        .route("/api/v1/auth", post(encrypted_request))
        .layer(DefaultBodyLimit::max(4096))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let http_addr: SocketAddr = config.bind_address.parse()?;

    match (
        config.tls_cert_path.as_deref(),
        config.tls_key_path.as_deref(),
    ) {
        (Some(cert), Some(key)) => {
            let tls_addr: SocketAddr = config.tls_bind_address.parse()?;
            let tls_config = RustlsConfig::from_pem_file(cert, key).await?;
            if config.tls_only {
                tracing::info!("Listening on https://{}", tls_addr);
                axum_server::bind_rustls(tls_addr, tls_config)
                    .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                    .await?;
            } else {
                tracing::info!("Listening on http://{} and https://{}", http_addr, tls_addr);
                let http_listener = tokio::net::TcpListener::bind(http_addr).await?;
                tokio::select! {
                    res = axum::serve(
                        http_listener,
                        app.clone().into_make_service_with_connect_info::<SocketAddr>(),
                    ) => res?,
                    res = axum_server::bind_rustls(tls_addr, tls_config)
                        .serve(app.into_make_service_with_connect_info::<SocketAddr>()) => res?,
                }
            }
        }
        (Some(_), None) => {
            anyhow::bail!("TLS_CERT_PATH is set but TLS_KEY_PATH is missing");
        }
        (None, Some(_)) => {
            anyhow::bail!("TLS_KEY_PATH is set but TLS_CERT_PATH is missing");
        }
        (None, None) => {
            if config.tls_only {
                anyhow::bail!(
                    "TLS_ONLY is set but TLS_CERT_PATH and TLS_KEY_PATH are not configured"
                );
            }
            tracing::warn!(
                "TLS is not configured. Set TLS_CERT_PATH and TLS_KEY_PATH to enable HTTPS."
            );
            tracing::info!("Listening on http://{}", http_addr);
            let listener = tokio::net::TcpListener::bind(http_addr).await?;
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await?;
        }
    }

    Ok(())
}
