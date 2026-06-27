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

use anyhow::Result;
use dnf_shared::crypto::TokenGenerator;
use dnf_shared::error::DnfError;
use dnf_shared::types::UserId;
use mysql_async::Pool;
use mysql_async::TxOpts;
use mysql_async::prelude::*;

fn validate_username(s: &str) -> Result<()> {
    if s.len() < 4 || s.len() > 32 {
        return Err(anyhow::Error::new(DnfError::InvalidUsername));
    }
    if !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(anyhow::Error::new(DnfError::InvalidUsername));
    }
    Ok(())
}

fn validate_md5(s: &str) -> Result<()> {
    if s.len() != 32 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(anyhow::Error::new(DnfError::InvalidPassword));
    }
    Ok(())
}

fn validate_qq(s: &str) -> Result<()> {
    if s.len() < 5 || s.len() > 12 || !s.chars().all(|c| c.is_ascii_digit()) {
        return Err(anyhow::Error::new(DnfError::InvalidQqNumber));
    }
    Ok(())
}

pub struct AuthService {
    pool: Pool,
    token_generator: TokenGenerator,
    initial_cera: u32,
    initial_cera_point: u32,
}

impl AuthService {
    pub fn new(
        pool: Pool,
        token_generator: TokenGenerator,
        initial_cera: u32,
        initial_cera_point: u32,
    ) -> Self {
        Self {
            pool,
            token_generator,
            initial_cera,
            initial_cera_point,
        }
    }

    /// Authenticate user and generate a login token.
    pub async fn login(
        &self,
        username: &str,
        password_md5: &str,
        _mac_address: &str,
        _ip_address: &str,
    ) -> Result<(String, UserId)> {
        validate_username(username)?;
        validate_md5(password_md5)?;
        let (uid, _) = self
            .find_user_by_credentials(username, password_md5)
            .await?;
        self.check_banned(uid).await?;
        let token = self.token_generator.generate_token(uid)?;
        self.reset_character_limit(uid).await;
        Ok((token, uid))
    }

    /// Register a new user account.
    ///
    /// The username uniqueness check and UID allocation are performed inside
    /// the transaction to reduce the TOCTOU race window.
    pub async fn register(
        &self,
        username: &str,
        password_md5: &str,
        qq_number: Option<&str>,
    ) -> Result<UserId> {
        validate_username(username)?;
        validate_md5(password_md5)?;
        if let Some(qq) = qq_number {
            validate_qq(qq)?;
        }
        let mut conn = self.pool.get_conn().await?;
        let mut tx = conn.start_transaction(TxOpts::default()).await?;

        let count: Option<(i64,)> = tx
            .exec_first(
                "SELECT COUNT(*) FROM d_taiwan.accounts WHERE accountname = ?",
                (username,),
            )
            .await?;
        if count.map(|(n,)| n).unwrap_or(0) > 0 {
            return Err(anyhow::Error::new(DnfError::UserExists));
        }

        let row: Option<(u32,)> = tx
            .exec_first(
                "SELECT UID FROM d_taiwan.accounts ORDER BY UID DESC LIMIT 1 FOR UPDATE",
                (),
            )
            .await?;
        let uid: UserId = row.map(|(uid,)| uid + 1).unwrap_or(1);

        tx.exec_drop(
            "INSERT INTO d_taiwan.accounts (UID, accountname, password, qq) VALUES (?, ?, ?, ?)",
            (uid, username, password_md5, qq_number),
        )
        .await?;

        tx.exec_drop(
            "INSERT INTO d_taiwan.limit_create_character (m_id) VALUES (?)",
            (uid,),
        )
        .await?;

        tx.exec_drop(
            "INSERT INTO d_taiwan.member_info (m_id, user_id) VALUES (?, ?)",
            (uid, uid),
        )
        .await?;

        tx.exec_drop(
            "INSERT INTO d_taiwan.member_white_account (m_id) VALUES (?)",
            (uid,),
        )
        .await?;

        tx.exec_drop(
            "INSERT INTO taiwan_login.member_login (m_id) VALUES (?)",
            (uid,),
        )
        .await?;

        tx.exec_drop(
            "INSERT INTO taiwan_billing.cash_cera \
             (account, cera, mod_tran, mod_date, reg_date) \
             VALUES (?, ?, 0, NOW(), NOW())",
            (uid, self.initial_cera),
        )
        .await?;

        tx.exec_drop(
            "INSERT INTO taiwan_billing.cash_cera_point \
             (account, cera_point, mod_date, reg_date) \
             VALUES (?, ?, NOW(), NOW())",
            (uid, self.initial_cera_point),
        )
        .await?;

        tx.exec_drop(
            "INSERT INTO taiwan_cain_2nd.member_avatar_coin (m_id) VALUES (?)",
            (uid,),
        )
        .await?;

        tx.commit().await?;

        self.insert_optional_member_tables(uid).await;

        tracing::info!("User registered: uid={}", uid);
        Ok(uid)
    }

    /// Best-effort inserts for supplemental tables that may not exist on all
    /// server configurations. Failures are silently ignored.
    async fn insert_optional_member_tables(&self, uid: UserId) {
        if let Ok(mut conn) = self.pool.get_conn().await {
            let _ = conn
                .exec_drop(
                    "INSERT INTO d_taiwan.member_join_info (m_id) VALUES (?)",
                    (uid,),
                )
                .await;
            let _ = conn
                .exec_drop(
                    "INSERT INTO d_taiwan.member_miles (m_id) VALUES (?)",
                    (uid,),
                )
                .await;
        }
    }

    /// Change password after verifying the old password.
    pub async fn change_password(
        &self,
        username: &str,
        old_password_md5: &str,
        new_password_md5: &str,
    ) -> Result<()> {
        validate_username(username)?;
        validate_md5(old_password_md5)?;
        validate_md5(new_password_md5)?;
        let (uid, _) = self
            .find_user_by_credentials(username, old_password_md5)
            .await?;
        let mut conn = self.pool.get_conn().await?;
        conn.exec_drop(
            "UPDATE d_taiwan.accounts SET password = ? WHERE UID = ?",
            (new_password_md5, uid),
        )
        .await?;
        Ok(())
    }

    /// Reset password after verifying the QQ number.
    pub async fn forgot_password(
        &self,
        username: &str,
        qq_number: &str,
        new_password_md5: &str,
    ) -> Result<()> {
        validate_username(username)?;
        validate_qq(qq_number)?;
        validate_md5(new_password_md5)?;
        let uid = self.find_uid_by_username_qq(username, qq_number).await?;
        let mut conn = self.pool.get_conn().await?;
        conn.exec_drop(
            "UPDATE d_taiwan.accounts SET password = ? WHERE UID = ?",
            (new_password_md5, uid),
        )
        .await?;
        Ok(())
    }

    async fn find_user_by_credentials(
        &self,
        username: &str,
        password_md5: &str,
    ) -> Result<(UserId, String)> {
        let mut conn = self.pool.get_conn().await?;
        let row: Option<(u32, String)> = conn
            .exec_first(
                "SELECT UID, accountname FROM d_taiwan.accounts \
             WHERE accountname = ? AND password = ?",
                (username, password_md5),
            )
            .await?;
        row.ok_or_else(|| anyhow::Error::new(DnfError::AuthenticationFailed))
    }

    async fn find_uid_by_username_qq(&self, username: &str, qq: &str) -> Result<UserId> {
        let mut conn = self.pool.get_conn().await?;
        let row: Option<(u32,)> = conn
            .exec_first(
                "SELECT UID FROM d_taiwan.accounts WHERE accountname = ? AND qq = ?",
                (username, qq),
            )
            .await?;
        row.map(|(uid,)| uid)
            .ok_or_else(|| anyhow::anyhow!("Username or QQ number not found"))
    }

    /// Checks whether the account has an active ban record.
    async fn check_banned(&self, uid: UserId) -> Result<()> {
        let mut conn = self.pool.get_conn().await?;
        let row: Option<(u32,)> = conn
            .exec_first(
                "SELECT m_id FROM d_taiwan.member_punish_info \
                 WHERE m_id = ? AND apply_flag = 1 AND end_time > NOW() \
                 LIMIT 1",
                (uid,),
            )
            .await?;
        if row.is_some() {
            tracing::info!("login blocked by active ban: uid={}", uid);
            return Err(anyhow::Error::new(DnfError::AccountBanned(
                "Account banned by administrator".to_string(),
            )));
        }
        Ok(())
    }

    /// Best-effort reset of the character creation counter. Failures are ignored.
    async fn reset_character_limit(&self, uid: UserId) {
        if let Ok(mut conn) = self.pool.get_conn().await {
            let _ = conn
                .exec_drop(
                    "UPDATE d_taiwan.limit_create_character SET count = 0 WHERE m_id = ?",
                    (uid,),
                )
                .await;
        }
    }
}
