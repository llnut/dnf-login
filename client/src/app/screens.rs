use eframe::egui;

use super::{AppState, DnfLoginApp, TaskType};
use crate::{
    config::BgFillMode,
    i18n::{Language, translations},
};

// Translations helper
impl DnfLoginApp {
    pub(super) fn t(&self) -> crate::i18n::Tr {
        self.tr
    }
}

// Login Screen
impl DnfLoginApp {
    pub(super) fn show_login_screen(&mut self, ui: &mut egui::Ui) {
        let tr = self.t();

        ui.vertical_centered(|ui| {
            Self::draw_app_icon(ui, self.app_icon.as_ref());
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(tr.app_title)
                    .size(22.0)
                    .strong()
                    .color(Self::c_text()),
            );
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(tr.app_subtitle)
                    .size(12.0)
                    .color(Self::c_text3()),
            );
        });

        ui.add_space(10.0);

        if !self.config.is_configured() {
            Self::warning_box(ui, tr.warn_server_not_configured);
            ui.add_space(12.0);
        }

        ui.add(Self::text_input(
            tr.username,
            &mut self.username,
            tr.hint_username,
        ));
        ui.add_space(12.0);

        ui.add(Self::password_input(
            tr.password,
            &mut self.password,
            tr.hint_password,
        ));
        ui.add_space(9.0);

        ui.checkbox(
            &mut self.remember_password,
            egui::RichText::new(tr.remember_password)
                .size(13.5)
                .color(Self::c_text2()),
        );
        ui.add_space(16.0);

        let login_enabled =
            !self.username.is_empty() && !self.password.is_empty() && self.current_task.is_none();
        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

        if Self::primary_button(ui, tr.enter_game, login_enabled)
            || (enter_pressed && login_enabled)
        {
            self.handle_login();
        }

        if matches!(self.current_task, Some(TaskType::Login)) {
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(
                        egui::RichText::new(tr.signing_in)
                            .size(13.0)
                            .color(Self::c_text2()),
                    );
                });
            });
        }

        ui.add_space(16.0);
        let sep = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 1.0));
        ui.painter()
            .rect_filled(sep, egui::CornerRadius::ZERO, Self::c_border());
        ui.add_space(11.0);

        ui.columns(4, |cols| {
            cols[0].vertical_centered(|ui| {
                if ui
                    .link(
                        egui::RichText::new(tr.register_link)
                            .size(13.0)
                            .color(Self::c_text2()),
                    )
                    .clicked()
                {
                    self.state = AppState::Register;
                    self.message = None;
                    self.message_is_error = false;
                }
            });
            cols[1].vertical_centered(|ui| {
                if ui
                    .link(
                        egui::RichText::new(tr.change_password_link)
                            .size(13.0)
                            .color(Self::c_text2()),
                    )
                    .clicked()
                {
                    self.state = AppState::ChangePassword;
                    self.message = None;
                    self.message_is_error = false;
                }
            });
            cols[2].vertical_centered(|ui| {
                if ui
                    .link(
                        egui::RichText::new(tr.settings_link)
                            .size(13.0)
                            .color(Self::c_text2()),
                    )
                    .clicked()
                {
                    self.state = AppState::Settings;
                    self.message = None;
                    self.message_is_error = false;
                    self.settings_server_url = self.config.server_url.clone();
                    self.settings_aes_key = self.config.aes_key.clone();
                    self.settings_bg_path = self.config.bg_custom_path.clone();
                }
            });
            cols[3].vertical_centered(|ui| {
                if ui
                    .link(
                        egui::RichText::new(tr.about_link)
                            .size(13.0)
                            .color(Self::c_text2()),
                    )
                    .clicked()
                {
                    self.state = AppState::About;
                    self.message = None;
                    self.message_is_error = false;
                }
            });
        });
    }
}

// Register Screen
impl DnfLoginApp {
    pub(super) fn show_register_screen(&mut self, ui: &mut egui::Ui) {
        let tr = self.t();

        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new(tr.create_account_title)
                    .size(20.0)
                    .strong()
                    .color(Self::c_text()),
            );
        });
        ui.add_space(14.0);

        ui.add(Self::text_input(
            tr.username,
            &mut self.register_username,
            tr.hint_choose_username,
        ));
        ui.add_space(12.0);

        ui.add(Self::password_input(
            tr.password,
            &mut self.register_password,
            tr.hint_choose_password,
        ));
        ui.add_space(12.0);

        ui.add(Self::password_input(
            tr.confirm_password,
            &mut self.register_password_confirm,
            tr.hint_re_enter_password,
        ));
        ui.add_space(12.0);

        ui.add(Self::text_input(
            tr.qq_optional,
            &mut self.register_qq,
            tr.hint_qq,
        ));
        ui.add_space(16.0);

        let sep = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 1.0));
        ui.painter()
            .rect_filled(sep, egui::CornerRadius::ZERO, Self::c_border());
        ui.add_space(13.0);

        ui.horizontal(|ui| {
            if ui.add(Self::secondary_button(tr.back)).clicked() {
                self.state = AppState::Login;
                self.message = None;
                self.message_is_error = false;
            }
            ui.add_space(10.0);

            let enabled = !self.register_username.is_empty()
                && !self.register_password.is_empty()
                && !self.register_password_confirm.is_empty()
                && self.current_task.is_none();
            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

            if Self::primary_button_slim(ui, tr.register_btn, enabled) || (enter_pressed && enabled)
            {
                self.handle_register();
            }

            if matches!(self.current_task, Some(TaskType::Register)) {
                ui.add_space(8.0);
                ui.spinner();
            }
        });
    }
}

// Change Password Screen
impl DnfLoginApp {
    pub(super) fn show_change_password_screen(&mut self, ui: &mut egui::Ui) {
        let tr = self.t();

        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new(tr.change_password_title)
                    .size(19.0)
                    .strong()
                    .color(Self::c_text()),
            );
        });
        ui.add_space(14.0);

        ui.add(Self::text_input(
            tr.username,
            &mut self.changepwd_username,
            tr.hint_username,
        ));
        ui.add_space(12.0);

        ui.add(Self::password_input(
            tr.current_password,
            &mut self.changepwd_old_password,
            tr.hint_current_password,
        ));
        ui.add_space(12.0);

        ui.add(Self::password_input(
            tr.new_password,
            &mut self.changepwd_new_password,
            tr.hint_enter_new_password,
        ));
        ui.add_space(12.0);

        ui.add(Self::password_input(
            tr.confirm_new_password,
            &mut self.changepwd_confirm,
            tr.hint_confirm_new_password,
        ));
        ui.add_space(16.0);

        let sep = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 1.0));
        ui.painter()
            .rect_filled(sep, egui::CornerRadius::ZERO, Self::c_border());
        ui.add_space(13.0);

        ui.horizontal(|ui| {
            if ui.add(Self::secondary_button(tr.back)).clicked() {
                self.state = AppState::Login;
                self.message = None;
                self.message_is_error = false;
            }
            ui.add_space(10.0);

            let enabled = !self.changepwd_username.is_empty()
                && !self.changepwd_old_password.is_empty()
                && !self.changepwd_new_password.is_empty()
                && !self.changepwd_confirm.is_empty()
                && self.current_task.is_none();
            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

            if Self::primary_button_slim(ui, tr.change_password_btn, enabled)
                || (enter_pressed && enabled)
            {
                self.handle_change_password();
            }

            if matches!(self.current_task, Some(TaskType::ChangePassword)) {
                ui.add_space(8.0);
                ui.spinner();
            }
        });
    }
}

// Settings Screen
impl DnfLoginApp {
    pub(super) fn show_settings_screen(&mut self, ui: &mut egui::Ui) {
        let tr = self.t();

        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new(tr.settings_title)
                    .size(19.0)
                    .strong()
                    .color(Self::c_text()),
            );
        });
        ui.add_space(12.0);

        egui::ScrollArea::vertical()
            .content_margin(egui::Margin::symmetric(0, 8))
            .show(ui, |ui| {
                if !self.config.is_configured() {
                    Self::warning_box(ui, tr.warn_first_launch);
                    ui.add_space(12.0);
                }

                ui.label(
                    egui::RichText::new(tr.language_label)
                        .size(13.0)
                        .color(Self::c_text2()),
                );
                ui.add_space(4.0);
                let old_lang = self.config.language;
                let avail = ui.available_width();
                egui::ComboBox::from_id_salt("language_select")
                    .selected_text(self.config.language.label())
                    .width(avail)
                    .show_ui(ui, |ui| {
                        for &lang in Language::all() {
                            ui.selectable_value(&mut self.config.language, lang, lang.label());
                        }
                    });
                if self.config.language != old_lang {
                    self.tr = translations(self.config.language);
                    let _ = self.config.save();
                }
                ui.add_space(14.0);

                ui.add(Self::text_input(
                    tr.server_url_label,
                    &mut self.settings_server_url,
                    tr.server_url_hint,
                ));
                ui.add_space(3.0);
                ui.label(
                    egui::RichText::new(tr.server_url_help)
                        .size(11.5)
                        .color(Self::c_text3()),
                );
                ui.add_space(14.0);

                ui.add(Self::text_input(
                    tr.aes_key_label,
                    &mut self.settings_aes_key,
                    tr.aes_key_hint,
                ));
                ui.add_space(3.0);
                ui.label(
                    egui::RichText::new(tr.aes_key_help)
                        .size(11.5)
                        .color(Self::c_text3()),
                );
                ui.add_space(14.0);

                let sep = egui::Rect::from_min_size(
                    ui.cursor().min,
                    egui::vec2(ui.available_width(), 1.0),
                );
                ui.painter()
                    .rect_filled(sep, egui::CornerRadius::ZERO, Self::c_border());
                ui.add_space(13.0);

                ui.add(Self::text_input(
                    tr.bg_custom_path_label,
                    &mut self.settings_bg_path,
                    tr.bg_custom_path_hint,
                ));
                ui.add_space(3.0);
                ui.label(
                    egui::RichText::new(tr.bg_custom_path_help)
                        .size(11.5)
                        .color(Self::c_text3()),
                );
                ui.add_space(10.0);

                ui.label(
                    egui::RichText::new(tr.bg_position_label)
                        .size(13.0)
                        .color(Self::c_text2()),
                );
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let old_prepend = self.config.bg_custom_prepend;
                    ui.radio_value(
                        &mut self.config.bg_custom_prepend,
                        false,
                        egui::RichText::new(tr.bg_position_append).size(13.5),
                    );
                    ui.add_space(8.0);
                    ui.radio_value(
                        &mut self.config.bg_custom_prepend,
                        true,
                        egui::RichText::new(tr.bg_position_prepend).size(13.5),
                    );
                    if self.config.bg_custom_prepend != old_prepend {
                        let _ = self.config.save();
                    }
                });
                ui.add_space(10.0);

                if Self::primary_button_slim(ui, tr.bg_reload_btn, true) {
                    self.config.bg_custom_path = self.settings_bg_path.trim().to_string();
                    let _ = self.config.save();
                    self.start_bg_loading();
                }

                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new(tr.bg_fill_mode_label)
                        .size(13.0)
                        .color(Self::c_text2()),
                );
                ui.add_space(4.0);
                let fill_mode_label = match &self.config.bg_fill_mode {
                    BgFillMode::Tile => tr.bg_fill_tile,
                    BgFillMode::Stretch => tr.bg_fill_stretch,
                    BgFillMode::Fill => tr.bg_fill_fill,
                    BgFillMode::Center => tr.bg_fill_center,
                    BgFillMode::Fit => tr.bg_fill_fit,
                };
                let avail = ui.available_width();
                let old_mode = self.config.bg_fill_mode;
                egui::ComboBox::from_id_salt("bg_fill_mode")
                    .selected_text(fill_mode_label)
                    .width(avail)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.config.bg_fill_mode,
                            BgFillMode::Fill,
                            tr.bg_fill_fill,
                        );
                        ui.selectable_value(
                            &mut self.config.bg_fill_mode,
                            BgFillMode::Fit,
                            tr.bg_fill_fit,
                        );
                        ui.selectable_value(
                            &mut self.config.bg_fill_mode,
                            BgFillMode::Stretch,
                            tr.bg_fill_stretch,
                        );
                        ui.selectable_value(
                            &mut self.config.bg_fill_mode,
                            BgFillMode::Center,
                            tr.bg_fill_center,
                        );
                        ui.selectable_value(
                            &mut self.config.bg_fill_mode,
                            BgFillMode::Tile,
                            tr.bg_fill_tile,
                        );
                    });
                if self.config.bg_fill_mode != old_mode {
                    let _ = self.config.save();
                }

                ui.add_space(14.0);

                let sep = egui::Rect::from_min_size(
                    ui.cursor().min,
                    egui::vec2(ui.available_width(), 1.0),
                );
                ui.painter()
                    .rect_filled(sep, egui::CornerRadius::ZERO, Self::c_border());
                ui.add_space(13.0);

                ui.checkbox(
                    &mut self.settings_plugin_inject_enabled,
                    egui::RichText::new(tr.plugin_inject_label)
                        .size(13.5)
                        .color(Self::c_text2()),
                );
                ui.add_space(3.0);
                ui.label(
                    egui::RichText::new(tr.plugin_inject_help)
                        .size(11.5)
                        .color(Self::c_text3()),
                );
                ui.add_space(10.0);

                ui.add(Self::text_input(
                    tr.plugins_path_label,
                    &mut self.settings_plugins_path,
                    tr.plugins_path_hint,
                ));
                ui.add_space(3.0);
                ui.label(
                    egui::RichText::new(tr.plugins_path_help)
                        .size(11.5)
                        .color(Self::c_text3()),
                );
                ui.add_space(10.0);

                ui.checkbox(
                    &mut self.settings_game_server_ip_enabled,
                    egui::RichText::new(tr.game_server_ip_label)
                        .size(13.5)
                        .color(Self::c_text2()),
                );
                ui.add_space(3.0);
                ui.label(
                    egui::RichText::new(tr.game_server_ip_help)
                        .size(11.5)
                        .color(Self::c_text3()),
                );

                if let Some(msg) = &self.message {
                    ui.add_space(12.0);
                    Self::status_box(ui, msg, self.message_is_error);
                }

                ui.add_space(16.0);

                let sep = egui::Rect::from_min_size(
                    ui.cursor().min,
                    egui::vec2(ui.available_width(), 1.0),
                );
                ui.painter()
                    .rect_filled(sep, egui::CornerRadius::ZERO, Self::c_border());
                ui.add_space(10.0);

                ui.label(
                    egui::RichText::new(tr.saved_config_label)
                        .size(11.5)
                        .color(Self::c_text2()),
                );
                ui.add_space(5.0);

                if self.config.is_configured() {
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgba_premultiplied(5, 5, 12, 200))
                        .stroke(egui::Stroke::new(1.0, Self::c_border()))
                        .corner_radius(egui::CornerRadius::same(6))
                        .inner_margin(egui::vec2(10.0, 8.0))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new("URL").size(11.5).color(Self::c_text3()),
                                );
                                ui.add_space(2.0);
                                ui.label(
                                    egui::RichText::new(&self.config.server_url)
                                        .size(12.5)
                                        .color(Self::c_text()),
                                );
                            });
                            ui.add_space(10.0);
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new("KEY").size(11.5).color(Self::c_text3()),
                                );
                                ui.add_space(2.0);
                                ui.label(
                                    egui::RichText::new(&self.config.aes_key)
                                        .size(12.5)
                                        .color(Self::c_text()),
                                );
                            });
                        });
                } else {
                    ui.label(
                        egui::RichText::new(tr.not_configured)
                            .size(13.0)
                            .color(Self::c_text3()),
                    );
                }

                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    if ui.add(Self::secondary_button(tr.back)).clicked() {
                        self.state = AppState::Login;
                        self.message = None;
                        self.message_is_error = false;
                    }
                    ui.add_space(6.0);
                    if ui.add(Self::secondary_button(tr.clear_btn)).clicked() {
                        self.settings_server_url.clear();
                        self.settings_aes_key.clear();
                    }
                    ui.add_space(6.0);
                    let save_enabled = !self.settings_server_url.trim().is_empty()
                        && !self.settings_aes_key.trim().is_empty();
                    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if Self::primary_button_slim(ui, tr.save_btn, save_enabled)
                        || (enter_pressed && save_enabled)
                    {
                        self.handle_save_settings();
                    }
                });
            });
    }
}

// About Screen
impl DnfLoginApp {
    pub(super) fn show_about_screen(&mut self, ui: &mut egui::Ui) {
        let tr = self.t();

        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new(tr.about_title)
                    .size(19.0)
                    .strong()
                    .color(Self::c_text()),
            );
        });
        ui.add_space(14.0);

        let sep = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 1.0));
        ui.painter()
            .rect_filled(sep, egui::CornerRadius::ZERO, Self::c_border());
        ui.add_space(15.0);

        let row_height = 22.0;
        let label_width = 90.0;

        let rows: &[(&str, &str, bool)] = &[
            (tr.about_launcher_name_label, "llnut launcher", false),
            (tr.about_version_label, env!("CARGO_PKG_VERSION"), false),
            (
                tr.about_repo_label,
                "https://github.com/llnut/dnf-login",
                true,
            ),
            (tr.about_author_label, "llnut", false),
        ];

        for &(label, value, is_link) in rows {
            ui.horizontal(|ui| {
                ui.add_sized(
                    egui::vec2(label_width, row_height),
                    egui::Label::new(egui::RichText::new(label).size(12.5).color(Self::c_text2())),
                );
                if is_link {
                    ui.hyperlink_to(
                        egui::RichText::new(value).size(13.5).color(Self::c_text()),
                        value,
                    );
                } else {
                    ui.label(egui::RichText::new(value).size(13.5).color(Self::c_text()));
                }
            });
            ui.add_space(6.0);
        }

        ui.add_space(10.0);
        let sep = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 1.0));
        ui.painter()
            .rect_filled(sep, egui::CornerRadius::ZERO, Self::c_border());
        ui.add_space(13.0);

        if ui
            .add_sized(
                egui::vec2(ui.available_width(), 36.0),
                Self::secondary_button(tr.back),
            )
            .clicked()
        {
            self.state = AppState::Login;
        }
    }
}

// Confirm close DNF dialog
impl DnfLoginApp {
    pub(super) fn show_confirm_close_dialog(&mut self, ctx: &egui::Context) {
        let tr = self.t();
        let loading = if matches!(self.current_task, Some(TaskType::CloseDnf)) {
            Some(tr.closing_dnf)
        } else {
            None
        };

        let action = Self::confirm_dialog(
            ctx,
            "close_dialog",
            tr.confirm_close_dnf_title,
            tr.confirm_close_dnf_msg,
            tr.confirm_close_yes,
            tr.confirm_close_no,
            loading,
        );

        match action {
            Some(true) => self.handle_close_dnf(),
            Some(false) => {
                self.show_confirm_close_dnf = false;
                self.pending_launch = None;
                self.message = None;
            }
            None => {}
        }
    }
}

// Global message overlay
impl DnfLoginApp {
    pub(super) fn show_message(&self, ui: &mut egui::Ui) {
        if self.state == AppState::Settings || self.state == AppState::About {
            return;
        }

        if let Some(msg) = &self.message {
            ui.add_space(10.0);
            let text_color = if self.message_is_error {
                Self::c_error()
            } else {
                Self::c_success()
            };
            egui::Frame::new()
                .fill(if self.message_is_error {
                    Self::c_error_bg()
                } else {
                    Self::c_success_bg()
                })
                .stroke(egui::Stroke::new(1.0, text_color))
                .corner_radius(egui::CornerRadius::same(6))
                .inner_margin(egui::vec2(16.0, 10.0))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(msg.as_str())
                                .size(13.5)
                                .color(text_color),
                        );
                    });
                });
        }
    }

    pub(super) fn set_error(&mut self, msg: impl Into<String>) {
        self.message = Some(msg.into());
        self.message_is_error = true;
    }

    pub(super) fn set_success(&mut self, msg: impl Into<String>) {
        self.message = Some(msg.into());
        self.message_is_error = false;
    }
}
