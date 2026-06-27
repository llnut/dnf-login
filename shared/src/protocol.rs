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

use super::types::*;

/// Client request types
#[derive(Debug, Clone)]
pub enum Request {
    Login {
        username: String,
        password_md5: String,
        mac_address: String,
    },
    Register {
        username: String,
        password_md5: String,
        qq_number: Option<String>,
    },
    ForgotPassword {
        username: String,
        qq_number: String,
        new_password_md5: String,
    },
    ChangePassword {
        username: String,
        old_password_md5: String,
        new_password_md5: String,
    },
}

impl Request {
    /// Parses the pipe-delimited wire format: `"command|param1|param2|..."`
    ///
    /// A field containing `|` would produce extra split parts, causing the
    /// `len() == 4` guard to reject it as `InvalidProtocol`. No separate
    /// pipe-validation step is therefore needed.
    pub fn parse(data: &str) -> super::Result<Self> {
        let parts: Vec<&str> = data.split('|').collect();

        if parts.is_empty() {
            return Err(super::DnfError::InvalidProtocol);
        }

        match parts[0] {
            "login" if parts.len() == 4 => Ok(Request::Login {
                username: parts[1].to_string(),
                password_md5: parts[2].to_string(),
                mac_address: parts[3].to_string(),
            }),
            "regedit" if parts.len() == 4 => Ok(Request::Register {
                username: parts[1].to_string(),
                password_md5: parts[2].to_string(),
                qq_number: if parts[3].is_empty() {
                    None
                } else {
                    Some(parts[3].to_string())
                },
            }),
            "forget" if parts.len() == 4 => Ok(Request::ForgotPassword {
                username: parts[1].to_string(),
                qq_number: parts[2].to_string(),
                new_password_md5: parts[3].to_string(),
            }),
            "repasswd" if parts.len() == 4 => Ok(Request::ChangePassword {
                username: parts[1].to_string(),
                old_password_md5: parts[2].to_string(),
                new_password_md5: parts[3].to_string(),
            }),
            _ => Err(super::DnfError::InvalidProtocol),
        }
    }

    /// Encodes the request to its pipe-delimited wire format.
    pub fn encode(&self) -> String {
        match self {
            Request::Login {
                username,
                password_md5,
                mac_address,
            } => {
                format!("login|{}|{}|{}", username, password_md5, mac_address)
            }
            Request::Register {
                username,
                password_md5,
                qq_number,
            } => {
                format!(
                    "regedit|{}|{}|{}",
                    username,
                    password_md5,
                    qq_number.as_deref().unwrap_or("")
                )
            }
            Request::ForgotPassword {
                username,
                qq_number,
                new_password_md5,
            } => {
                format!("forget|{}|{}|{}", username, qq_number, new_password_md5)
            }
            Request::ChangePassword {
                username,
                old_password_md5,
                new_password_md5,
            } => {
                format!(
                    "repasswd|{}|{}|{}",
                    username, old_password_md5, new_password_md5
                )
            }
        }
    }
}

/// Server response data variants
#[derive(Debug, Clone)]
pub enum ResponseData {
    LoginSuccess { token: String, user_id: UserId },
    RegisterSuccess,
    Success,
    Error { error: String },
}

/// Server response wrapper
#[derive(Debug, Clone)]
pub struct Response {
    pub success: bool,
    pub data: ResponseData,
}

impl Response {
    pub fn login_success(token: String, user_id: UserId) -> Self {
        Self {
            success: true,
            data: ResponseData::LoginSuccess { token, user_id },
        }
    }

    pub fn register_success() -> Self {
        Self {
            success: true,
            data: ResponseData::RegisterSuccess,
        }
    }

    pub fn success() -> Self {
        Self {
            success: true,
            data: ResponseData::Success,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: ResponseData::Error {
                error: message.into(),
            },
        }
    }

    /// Encodes the response to its wire format.
    ///
    /// Login success yields `"0|<token>"`; other success yields `"success"`;
    /// error yields the error message string.
    pub fn encode(&self) -> String {
        match &self.data {
            ResponseData::LoginSuccess { token, .. } => format!("0|{}", token),
            ResponseData::RegisterSuccess => "success".to_string(),
            ResponseData::Success => "success".to_string(),
            ResponseData::Error { error } => error.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_parse_login() {
        let req =
            Request::parse("login|testuser|5f4dcc3b5aa765d61d8327deb882cf99|00:11:22:33:44:55")
                .unwrap();
        if let Request::Login {
            username,
            password_md5,
            mac_address,
        } = req
        {
            assert_eq!(username, "testuser");
            assert_eq!(password_md5, "5f4dcc3b5aa765d61d8327deb882cf99");
            assert_eq!(mac_address, "00:11:22:33:44:55");
        } else {
            panic!("Wrong request type");
        }
    }

    #[test]
    fn test_request_encode() {
        let req = Request::Login {
            username: "testuser".to_string(),
            password_md5: "abc123".to_string(),
            mac_address: "00:11:22:33:44:55".to_string(),
        };
        assert_eq!(req.encode(), "login|testuser|abc123|00:11:22:33:44:55");
    }
}
