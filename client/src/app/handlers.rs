use dnf_shared::error::error_key;

use super::{DnfLoginApp, PendingLaunch, TaskResult, TaskType};
use crate::{
    config::AppConfig,
    game::DnfLauncher,
    i18n::translations,
    network::{DnfClient, md5_hash},
    platform,
};

// Action handlers
impl DnfLoginApp {
    fn translate_server_error(&self, error: &str) -> &'static str {
        let tr = self.t();
        match error {
            error_key::INVALID_USERNAME => tr.err_server_invalid_username,
            error_key::INVALID_PASSWORD => tr.err_server_invalid_password,
            error_key::INVALID_QQ => tr.err_server_invalid_qq,
            error_key::USER_EXISTS => tr.err_server_user_exists,
            error_key::WRONG_CREDENTIALS => tr.err_server_wrong_credentials,
            error_key::ACCOUNT_BANNED => tr.err_server_account_banned,
            error_key::REGISTRATION_CLOSED => tr.err_server_registration_closed,
            _ => {
                tracing::warn!("Unknown server error key: {}", error);
                tr.err_server_unknown
            }
        }
    }

    pub(super) fn handle_login(&mut self) {
        self.message = None;
        self.message_is_error = false;
        let tr = self.t();

        if !self.config.is_configured() {
            self.set_error(tr.err_server_not_configured);
            return;
        }
        if self.username.is_empty() || self.password.is_empty() {
            self.set_error(tr.err_enter_username_password);
            return;
        }

        let mac_address = match platform::get_mac_address() {
            Ok(mac) => mac,
            Err(e) => {
                self.set_error(format!("{}: {}", tr.err_mac_prefix, e));
                return;
            }
        };

        let password_md5 = md5_hash(&self.password);

        let client = match self.client.clone() {
            Some(c) => c,
            None => {
                self.set_error(tr.err_client_not_init);
                return;
            }
        };

        self.current_task = Some(TaskType::Login);

        let username = self.username.clone();
        let tx = self.task_tx.clone();
        let inject_enabled = self.config.plugin_inject_enabled;
        let ip_enabled = self.config.game_server_ip_enabled;
        let server_url = self.config.server_url.clone();

        tracing::info!(
            "Login task started: user={}, mac={}",
            self.username,
            mac_address
        );

        self.runtime.spawn(async move {
            let (game_server_ip, result) = if inject_enabled && ip_enabled {
                let (ip_res, resolve_res, login_res) = tokio::join!(
                    client.get_game_server_ip(),
                    crate::network::resolve_server_ip(&server_url),
                    client.login(&username, &password_md5, &mac_address)
                );
                let ip = match ip_res {
                    Ok(Some(ip)) => Some(ip),
                    Ok(None) => resolve_res,
                    Err(e) => {
                        tracing::warn!("Failed to fetch game server IP: {}", e);
                        resolve_res
                    }
                };
                (ip, login_res)
            } else {
                (
                    None,
                    client.login(&username, &password_md5, &mac_address).await,
                )
            };
            let _ = tx.send(TaskResult::Login(result, game_server_ip));
        });
    }

    pub(super) fn handle_register(&mut self) {
        self.message = None;
        self.message_is_error = false;
        let tr = self.t();

        if !self.config.is_configured() {
            self.set_error(tr.err_server_not_configured);
            return;
        }
        if self.register_username.is_empty() {
            self.set_error(tr.err_enter_username);
            return;
        }
        if self.register_password.is_empty() {
            self.set_error(tr.err_enter_password);
            return;
        }
        if self.register_password != self.register_password_confirm {
            self.set_error(tr.err_passwords_no_match);
            return;
        }

        let password_md5 = md5_hash(&self.register_password);
        let qq = if self.register_qq.is_empty() {
            None
        } else {
            Some(self.register_qq.clone())
        };

        let client = match self.client.clone() {
            Some(c) => c,
            None => {
                self.set_error(tr.err_client_not_init);
                return;
            }
        };

        self.current_task = Some(TaskType::Register);

        let username = self.register_username.clone();
        let tx = self.task_tx.clone();

        tracing::info!("Register task started: user={}", self.register_username);

        self.runtime.spawn(async move {
            let result = client.register(&username, &password_md5, qq).await;
            let _ = tx.send(TaskResult::Register(result));
        });
    }

    pub(super) fn handle_change_password(&mut self) {
        self.message = None;
        self.message_is_error = false;
        let tr = self.t();

        if !self.config.is_configured() {
            self.set_error(tr.err_server_not_configured);
            return;
        }
        if self.changepwd_username.is_empty() {
            self.set_error(tr.err_enter_username);
            return;
        }
        if self.changepwd_old_password.is_empty() {
            self.set_error(tr.err_enter_old_password);
            return;
        }
        if self.changepwd_new_password.is_empty() {
            self.set_error(tr.err_enter_new_password);
            return;
        }
        if self.changepwd_new_password != self.changepwd_confirm {
            self.set_error(tr.err_passwords_no_match);
            return;
        }

        let old_md5 = md5_hash(&self.changepwd_old_password);
        let new_md5 = md5_hash(&self.changepwd_new_password);

        let client = match self.client.clone() {
            Some(c) => c,
            None => {
                self.set_error(tr.err_client_not_init);
                return;
            }
        };

        self.current_task = Some(TaskType::ChangePassword);

        let username = self.changepwd_username.clone();
        let tx = self.task_tx.clone();

        self.runtime.spawn(async move {
            let result = client.change_password(&username, &old_md5, &new_md5).await;
            let _ = tx.send(TaskResult::ChangePassword(result));
        });

        tracing::info!(
            "Change password task started: user={}",
            self.changepwd_username
        );
    }

    pub(super) fn handle_save_settings(&mut self) {
        self.message = None;
        self.message_is_error = false;
        let tr = self.t();

        let new_config = AppConfig {
            server_url: self.settings_server_url.trim().to_string(),
            aes_key: self.settings_aes_key.trim().to_string(),
            plugins_path: self.settings_plugins_path.trim().to_string(),
            plugin_inject_enabled: self.settings_plugin_inject_enabled,
            game_server_ip_enabled: self.settings_game_server_ip_enabled,
            ..self.config.clone()
        };

        if let Err(e) = new_config.validate() {
            self.set_error(format!("{}: {}", tr.err_config_prefix, e));
            return;
        }
        if let Err(e) = new_config.save() {
            self.set_error(format!("{}: {}", tr.err_save_prefix, e));
            return;
        }

        self.config = new_config;
        self.settings_server_url = self.config.server_url.clone();
        self.settings_aes_key = self.config.aes_key.clone();
        self.tr = translations(self.config.language);

        match self.config.get_aes_key_bytes() {
            Ok(key) => match DnfClient::new(self.config.server_url.clone(), &key) {
                Ok(client) => {
                    self.client = Some(client);
                    self.set_success(tr.settings_saved);
                }
                Err(e) => self.set_error(format!("Failed to initialize network client: {}", e)),
            },
            Err(e) => self.set_error(format!("Failed to parse AES key: {}", e)),
        }
    }

    pub(super) fn handle_task_result(&mut self, result: TaskResult) {
        let tr = self.t();
        match result {
            TaskResult::Login(res, game_server_ip) => {
                self.current_task = None;
                match res {
                    Ok(response) => {
                        if response.success {
                            if let Some(token) = response.token {
                                if let Err(e) = self.storage.save(
                                    &self.username,
                                    &self.password,
                                    self.remember_password,
                                ) {
                                    tracing::warn!("Failed to save credentials: {}", e);
                                }
                                // Clear the in-memory plaintext password.
                                self.password = String::new();
                                // Restore from storage now so re-launch works regardless of
                                // whether the launch call below succeeds or fails.
                                if self.remember_password
                                    && let Ok((_, p)) = self.storage.load()
                                {
                                    self.password = p;
                                }
                                self.set_success(tr.login_success);

                                let plugins_path = self.config.plugins_path.clone();
                                let inject_enabled = self.config.plugin_inject_enabled;
                                let server_ip = game_server_ip.unwrap_or_default();

                                if platform::is_process_running("DNF.exe").unwrap_or(false) {
                                    self.pending_launch = Some(PendingLaunch {
                                        token,
                                        plugins_path,
                                        inject_enabled,
                                        server_ip,
                                    });
                                    self.show_confirm_close_dnf = true;
                                    return;
                                }

                                if let Err(e) = DnfLauncher::launch_with_token(
                                    &token,
                                    &plugins_path,
                                    inject_enabled,
                                    &server_ip,
                                ) {
                                    self.set_error(format!("{}: {}", tr.err_launch_prefix, e));
                                } else {
                                    tracing::info!("Game launched: user={}", self.username);
                                }
                            } else {
                                // Server reported success but returned no token; treat as an error.
                                tracing::error!("Login response: success=true but token is absent");
                                self.set_error(tr.err_network_prefix.to_string());
                            }
                        } else {
                            let msg = response.error.as_deref().unwrap_or("fail");
                            self.set_error(self.translate_server_error(msg));
                        }
                    }
                    Err(e) => self.set_error(format!("{}: {}", tr.err_network_prefix, e)),
                }
            }
            TaskResult::Register(res) => {
                self.current_task = None;
                match res {
                    Ok(response) => {
                        if response.success {
                            self.set_success(tr.register_success);
                            self.register_username.clear();
                            self.register_password.clear();
                            self.register_password_confirm.clear();
                            self.register_qq.clear();
                        } else {
                            let msg = response.error.as_deref().unwrap_or("fail");
                            self.set_error(self.translate_server_error(msg));
                        }
                    }
                    Err(e) => self.set_error(format!("{}: {}", tr.err_network_prefix, e)),
                }
            }
            TaskResult::ChangePassword(res) => {
                self.current_task = None;
                match res {
                    Ok(response) => {
                        if response.success {
                            self.set_success(tr.change_password_success);
                            self.changepwd_username.clear();
                            self.changepwd_old_password.clear();
                            self.changepwd_new_password.clear();
                            self.changepwd_confirm.clear();
                        } else {
                            let msg = response.error.as_deref().unwrap_or("fail");
                            self.set_error(self.translate_server_error(msg));
                        }
                    }
                    Err(e) => self.set_error(format!("{}: {}", tr.err_network_prefix, e)),
                }
            }
            TaskResult::CloseDnf(res) => {
                self.current_task = None;
                self.show_confirm_close_dnf = false;
                match res {
                    Ok(()) => {
                        if let Some(launch) = self.pending_launch.take() {
                            if let Err(e) = DnfLauncher::launch_with_token(
                                &launch.token,
                                &launch.plugins_path,
                                launch.inject_enabled,
                                &launch.server_ip,
                            ) {
                                self.set_error(format!("{}: {}", tr.err_launch_prefix, e));
                            } else {
                                tracing::info!("Game launched after closing previous instance");
                            }
                        }
                    }
                    Err(e) => {
                        self.pending_launch = None;
                        self.set_error(format!("{}: {}", tr.err_launch_prefix, e));
                    }
                }
            }
        }
    }

    pub(super) fn handle_close_dnf(&mut self) {
        tracing::info!("Closing DNF.exe before relaunch");
        self.current_task = Some(TaskType::CloseDnf);
        let tx = self.task_tx.clone();
        self.runtime.spawn(async move {
            let result =
                tokio::task::spawn_blocking(|| platform::graceful_terminate_process("DNF.exe"))
                    .await
                    .unwrap_or_else(|e| Err(anyhow::anyhow!("{}", e)));
            let _ = tx.send(TaskResult::CloseDnf(result));
        });
    }
}
