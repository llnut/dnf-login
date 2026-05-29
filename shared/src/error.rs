use thiserror::Error;

/// DNF login system error types
#[derive(Error, Debug)]
pub enum DnfError {
    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Token generation error: {0}")]
    TokenGeneration(String),

    #[error("Invalid hex string: {0}")]
    InvalidHex(#[from] hex::FromHexError),

    #[error("Invalid base64: {0}")]
    InvalidBase64(#[from] base64::DecodeError),

    #[error("RSA error: {0}")]
    Rsa(#[from] rsa::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid protocol data")]
    InvalidProtocol,

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserExists,

    #[error("Account banned: {0}")]
    AccountBanned(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid username")]
    InvalidUsername,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Invalid QQ number")]
    InvalidQqNumber,
}

/// Wire protocol error keys
pub mod error_key {
    pub const INVALID_USERNAME: &str = "invalid_username";
    pub const INVALID_PASSWORD: &str = "invalid_password";
    pub const INVALID_QQ: &str = "invalid_qq";
    pub const USER_EXISTS: &str = "user_exists";
    pub const WRONG_CREDENTIALS: &str = "wrong_credentials";
    pub const ACCOUNT_BANNED: &str = "account_banned";
    pub const REGISTRATION_CLOSED: &str = "registration_closed";
    pub const FAIL: &str = "fail";
}

/// Result type alias
pub type Result<T> = std::result::Result<T, DnfError>;
