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

use crate::services::AuthService;
use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use dnf_shared::{Request, error::DnfError, error::error_key, protocol::Response as DnfResponse};
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const RATE_LIMIT_MAX: u32 = 10;
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub aes_cipher: Arc<dnf_shared::crypto::AesGcmCipher>,
    pub rate_limiter: Arc<Mutex<HashMap<IpAddr, (u32, Instant)>>>,
    pub game_server_ip: Option<String>,
    pub registration_open: bool,
}

/// Main request handler for encrypted requests
pub async fn encrypted_request(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    body: String,
) -> Response {
    let ip = peer.ip();
    let ip_address = ip.to_string();

    {
        let mut limiter = state.rate_limiter.lock().unwrap();
        let now = Instant::now();
        let entry = limiter.entry(ip).or_insert((0, now));
        if now.duration_since(entry.1) >= RATE_LIMIT_WINDOW {
            *entry = (1, now);
        } else {
            entry.0 += 1;
            if entry.0 > RATE_LIMIT_MAX {
                tracing::warn!("Rate limit exceeded for IP: {}", ip);
                return (StatusCode::TOO_MANY_REQUESTS, "Too many requests").into_response();
            }
        }
    }

    let plaintext = match state.aes_cipher.decrypt_string(&body) {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Decryption failed: {}", e);
            return (StatusCode::BAD_REQUEST, "Decryption failed").into_response();
        }
    };

    let request = match Request::parse(&plaintext) {
        Ok(req) => req,
        Err(e) => {
            tracing::error!("Invalid request format: {}", e);
            return (StatusCode::BAD_REQUEST, "Invalid request").into_response();
        }
    };

    let response = match request {
        Request::Login {
            username,
            password_md5,
            mac_address,
        } => login(&state, &username, &password_md5, &mac_address, &ip_address).await,
        Request::Register {
            username,
            password_md5,
            qq_number,
        } => register(&state, &username, &password_md5, qq_number.as_deref()).await,
        Request::ForgotPassword {
            username,
            qq_number,
            new_password_md5,
        } => forgot_password(&state, &username, &qq_number, &new_password_md5).await,
        Request::ChangePassword {
            username,
            old_password_md5,
            new_password_md5,
        } => change_password(&state, &username, &old_password_md5, &new_password_md5).await,
    };

    let response_text = response.encode();
    let encrypted = match state.aes_cipher.encrypt_string(&response_text) {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Encryption failed: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Encryption failed").into_response();
        }
    };

    encrypted.into_response()
}

async fn login(
    state: &AppState,
    username: &str,
    password_md5: &str,
    mac_address: &str,
    ip_address: &str,
) -> DnfResponse {
    match state
        .auth_service
        .login(username, password_md5, mac_address, ip_address)
        .await
    {
        Ok((token, user_id)) => {
            tracing::info!(
                "User logged in: {} (uid={}, ip={})",
                username,
                user_id,
                ip_address
            );
            DnfResponse::login_success(token, user_id)
        }
        Err(e) => {
            tracing::warn!("Login failed for {} (ip={}): {}", username, ip_address, e);
            match e.downcast_ref::<DnfError>() {
                Some(DnfError::AccountBanned(_)) => DnfResponse::error(error_key::ACCOUNT_BANNED),
                _ => DnfResponse::error(error_key::WRONG_CREDENTIALS),
            }
        }
    }
}

async fn register(
    state: &AppState,
    username: &str,
    password_md5: &str,
    qq_number: Option<&str>,
) -> DnfResponse {
    if !state.registration_open {
        tracing::warn!(
            "Registration rejected for {}: registration is closed",
            username
        );
        return DnfResponse::error(error_key::REGISTRATION_CLOSED);
    }
    match state
        .auth_service
        .register(username, password_md5, qq_number)
        .await
    {
        Ok(user_id) => {
            tracing::info!("User registered: {} (uid={})", username, user_id);
            DnfResponse::register_success()
        }
        Err(e) => {
            tracing::warn!("Registration failed for {}: {}", username, e);
            match e.downcast_ref::<DnfError>() {
                Some(DnfError::UserExists) => DnfResponse::error(error_key::USER_EXISTS),
                Some(DnfError::InvalidUsername) => DnfResponse::error(error_key::INVALID_USERNAME),
                Some(DnfError::InvalidPassword) => DnfResponse::error(error_key::INVALID_PASSWORD),
                Some(DnfError::InvalidQqNumber) => DnfResponse::error(error_key::INVALID_QQ),
                _ => DnfResponse::error(error_key::FAIL),
            }
        }
    }
}

async fn forgot_password(
    state: &AppState,
    username: &str,
    qq_number: &str,
    new_password_md5: &str,
) -> DnfResponse {
    match state
        .auth_service
        .forgot_password(username, qq_number, new_password_md5)
        .await
    {
        Ok(_) => {
            tracing::info!("Password reset for user: {}", username);
            DnfResponse::success()
        }
        Err(e) => {
            tracing::warn!("Password reset failed for {}: {}", username, e);
            DnfResponse::error(error_key::FAIL)
        }
    }
}

async fn change_password(
    state: &AppState,
    username: &str,
    old_password_md5: &str,
    new_password_md5: &str,
) -> DnfResponse {
    match state
        .auth_service
        .change_password(username, old_password_md5, new_password_md5)
        .await
    {
        Ok(_) => {
            tracing::info!("Password changed for user: {}", username);
            DnfResponse::success()
        }
        Err(e) => {
            tracing::warn!("Password change failed for {}: {}", username, e);
            DnfResponse::error(error_key::FAIL)
        }
    }
}

/// Health check endpoint
pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Returns the game server IP as plain text, or 404 if not configured.
pub async fn game_server_ip(State(state): State<AppState>) -> impl IntoResponse {
    match &state.game_server_ip {
        Some(ip) => (StatusCode::OK, ip.clone()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
