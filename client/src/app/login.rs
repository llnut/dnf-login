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

use eframe::egui;

use super::{DnfLoginApp, TaskType};
use crate::i18n::Tr;

const HEADER_HEIGHT: f32 = 46.0;
const FOOTER_HEIGHT: f32 = 100.0;

pub(super) fn card_height(viewport_height: f32) -> f32 {
    // Leave the background switcher visible below the centered card.
    (viewport_height - 124.0).clamp(280.0, 410.0)
}

fn login_body(ui: &mut egui::Ui, height: f32, contents: impl FnOnce(&mut egui::Ui)) {
    egui::ScrollArea::vertical()
        .id_salt("login_form")
        .max_height(height)
        .min_scrolled_height(0.0)
        .auto_shrink([false, false])
        .show(ui, contents);
}

fn login_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    ui.add_enabled_ui(enabled, |ui| {
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 40.0), egui::Sense::click());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label)
        });
        let fill = if !ui.is_enabled() {
            egui::Color32::from_rgb(119, 132, 154)
        } else if response.is_pointer_button_down_on() {
            egui::Color32::from_rgb(188, 205, 230)
        } else if response.hovered() {
            egui::Color32::WHITE
        } else {
            egui::Color32::from_rgb(232, 239, 250)
        };
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(20), fill);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(14.0),
            egui::Color32::from_rgb(30, 42, 61),
        );
        response
    })
    .inner
}

fn status_line(ui: &mut egui::Ui, text: &str, error: bool, details: Option<&str>) -> bool {
    let mut clicked = false;
    ui.allocate_ui(egui::vec2(ui.available_width(), 22.0), |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(label) = details {
                clicked = ui.small_button(label).clicked();
            }
            ui.add(
                egui::Label::new(
                    egui::RichText::new(text.lines().next().unwrap_or_default())
                        .size(11.5)
                        .color(if error {
                            DnfLoginApp::c_error()
                        } else {
                            DnfLoginApp::c_text3()
                        }),
                )
                .truncate(),
            );
        });
    });
    clicked
}

impl DnfLoginApp {
    pub(super) fn show_login_screen(&mut self, ui: &mut egui::Ui) {
        let tr = self.t();
        let height = card_height(ui.ctx().viewport_rect().height());
        ui.spacing_mut().item_spacing.y = 4.0;
        let widgets = &mut ui.visuals_mut().widgets;
        widgets.inactive.bg_fill = egui::Color32::from_rgb(55, 73, 100);
        widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 125, 160));
        ui.set_min_height(height);
        ui.allocate_ui(egui::vec2(ui.available_width(), HEADER_HEIGHT), |ui| {
            ui.label(
                egui::RichText::new(tr.login_title)
                    .size(21.0)
                    .strong()
                    .color(Self::c_text()),
            );
            ui.add(
                egui::Label::new(
                    egui::RichText::new(tr.login_subtitle)
                        .size(12.0)
                        .color(Self::c_text3()),
                )
                .truncate(),
            );
        });

        let busy = self.current_task.is_some() || self.show_confirm_close_dnf;
        login_body(ui, height - HEADER_HEIGHT - FOOTER_HEIGHT - 8.0, |ui| {
            ui.add_enabled_ui(!busy, |ui| {
                self.show_saved_account_cards(ui);
                ui.vertical_centered(|ui| {
                    if ui.small_button(tr.new_account).clicked() {
                        self.username.clear();
                        self.password.clear();
                        self.remember_password = true;
                        self.message = None;
                    }
                });
                Self::login_input(ui, tr.username, &mut self.username, tr.hint_username, false);
                ui.add_space(3.0);
                Self::login_input(ui, tr.password, &mut self.password, tr.hint_password, true);
                ui.add_space(3.0);
                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut self.remember_password,
                        egui::RichText::new(tr.remember_password).size(12.0),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                !self.username.trim().is_empty() && !self.password.is_empty(),
                                egui::Button::new(tr.save_btn).small(),
                            )
                            .clicked()
                        {
                            match self.storage.save(
                                &self.username,
                                &self.password,
                                self.remember_password,
                            ) {
                                Ok(()) => {
                                    self.saved_accounts =
                                        self.storage.load_accounts().unwrap_or_default();
                                    if self.remember_password && self.config.allow_multi_instance {
                                        self.selected_accounts.insert(self.username.clone());
                                    } else {
                                        self.selected_accounts.remove(&self.username);
                                    }
                                }
                                Err(error) => self.set_error(error.to_string()),
                            }
                        }
                    });
                });
            });
        });

        // The footer is outside both scroll areas. Neither saved accounts nor
        // long launch results can move the launch button out of the card.
        ui.add_space(6.0);
        let count = self.login_accounts().len();
        let label = if matches!(self.current_task, Some(TaskType::Login)) {
            tr.signing_in.to_owned()
        } else if self.config.allow_multi_instance && !self.selected_accounts.is_empty() {
            format!("{} ({count})", tr.launch_selected)
        } else {
            tr.enter_game.to_owned()
        };
        let enabled = !busy && count > 0;
        let response = login_button(ui, &label, enabled);
        let enter = ui.input(|input| input.key_pressed(egui::Key::Enter));
        if response.clicked() || (enter && enabled && !self.show_login_details) {
            if self.config.allow_multi_instance {
                self.handle_batch_login();
            } else {
                self.handle_login();
            }
        }
        let status = if busy {
            tr.signing_in
        } else if let Some(message) = self.message.as_deref() {
            message
        } else if !self.config.is_configured() {
            tr.warn_server_not_configured
        } else {
            tr.ready
        };
        if status_line(
            ui,
            status,
            self.message_is_error,
            self.message.as_ref().map(|_| tr.view_details),
        ) {
            self.show_login_details = true;
        }
        ui.add_enabled_ui(!busy, |ui| self.show_login_links(ui));
    }

    fn login_input(ui: &mut egui::Ui, label: &str, value: &mut String, hint: &str, password: bool) {
        ui.label(egui::RichText::new(label).size(12.0).color(Self::c_text2()));
        ui.scope(|ui| {
            let widgets = &mut ui.visuals_mut().widgets;
            for visual in [
                &mut widgets.inactive,
                &mut widgets.hovered,
                &mut widgets.active,
            ] {
                visual.corner_radius = egui::CornerRadius::same(9);
            }
            ui.add(
                egui::TextEdit::singleline(value)
                    .password(password)
                    .desired_width(f32::INFINITY)
                    .min_size(egui::vec2(0.0, 20.0))
                    .margin(egui::vec2(10.0, 6.0))
                    .background_color(egui::Color32::from_rgba_unmultiplied(155, 180, 220, 20))
                    .hint_text(hint),
            );
        });
    }

    fn show_saved_account_cards(&mut self, ui: &mut egui::Ui) {
        let tr = self.t();
        let mut fill_account = None;
        let mut remove_account = None;
        egui::ScrollArea::vertical()
            .id_salt("saved_accounts")
            .max_height(88.0)
            .min_scrolled_height(0.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if self.saved_accounts.is_empty() {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(tr.no_saved_accounts)
                                .size(12.0)
                                .color(Self::c_text3()),
                        )
                        .truncate(),
                    );
                }
                for (name, password) in &self.saved_accounts {
                    ui.push_id(name, |ui| {
                        let mut selected = self.selected_accounts.contains(name);
                        let action = account_card(
                            ui,
                            name,
                            self.username == *name,
                            &mut selected,
                            self.config.allow_multi_instance,
                            tr,
                        );
                        if action.0 {
                            fill_account = Some((name.clone(), password.clone()));
                        }
                        if action.1 {
                            remove_account = Some(name.clone());
                        }
                        if selected {
                            self.selected_accounts.insert(name.clone());
                        } else {
                            self.selected_accounts.remove(name);
                        }
                    });
                }
            });
        if let Some((name, password)) = fill_account {
            self.username = name;
            self.password = password;
            self.remember_password = true;
        }
        if let Some(name) = remove_account {
            match self.storage.remove(&name) {
                Ok(()) => {
                    self.saved_accounts.retain(|(account, _)| account != &name);
                    self.selected_accounts.remove(&name);
                    if self.username == name {
                        self.username.clear();
                        self.password.clear();
                    }
                }
                Err(error) => self.set_error(error.to_string()),
            }
        }
    }

    pub(super) fn show_login_details_dialog(&mut self, ctx: &egui::Context) {
        let tr = self.t();
        egui::Window::new(tr.launch_details)
            .id(egui::Id::new("login_details"))
            .open(&mut self.show_login_details)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .order(egui::Order::Foreground)
            .fixed_size(egui::vec2(400.0, 220.0))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("login_result_details")
                    .show(ui, |ui| {
                        ui.add(
                            egui::Label::new(self.message.as_deref().unwrap_or(tr.ready)).wrap(),
                        );
                    });
            });
    }
}

// Clicking the account fills the form; the checkbox only changes the batch.
fn account_card(
    ui: &mut egui::Ui,
    name: &str,
    current: bool,
    selected: &mut bool,
    multi: bool,
    tr: Tr,
) -> (bool, bool) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 40.0), egui::Sense::hover());
    let name_rect = egui::Rect::from_min_max(
        rect.min,
        egui::pos2(rect.right() - if multi { 36.0 } else { 0.0 }, rect.bottom()),
    );
    let response = ui.interact(
        name_rect,
        ui.id().with("fill_account"),
        egui::Sense::click(),
    );
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), name));
    let mut remove = false;
    response.context_menu(|ui| {
        if ui.button(tr.remove_account).clicked() {
            remove = true;
            ui.close();
        }
    });
    let fill = if response.hovered() || current {
        egui::Color32::from_rgba_unmultiplied(160, 186, 224, 38)
    } else {
        egui::Color32::from_rgba_unmultiplied(130, 155, 198, 18)
    };
    ui.painter().rect_filled(rect, 9, fill);
    let avatar = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 21.0, rect.center().y),
        egui::vec2(26.0, 26.0),
    );
    ui.painter()
        .rect_filled(avatar, 8, egui::Color32::from_rgb(220, 230, 245));
    ui.painter().text(
        avatar.center(),
        egui::Align2::CENTER_CENTER,
        name.chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string(),
        egui::FontId::proportional(14.0),
        egui::Color32::from_rgb(35, 48, 68),
    );
    let painter = ui.painter().with_clip_rect(egui::Rect::from_min_max(
        egui::pos2(rect.left() + 43.0, rect.top()),
        egui::pos2(name_rect.right() - 4.0, rect.bottom()),
    ));
    painter.text(
        egui::pos2(rect.left() + 43.0, rect.top() + 5.0),
        egui::Align2::LEFT_TOP,
        name,
        egui::FontId::proportional(13.0),
        DnfLoginApp::c_text(),
    );
    painter.text(
        egui::pos2(rect.left() + 43.0, rect.top() + 23.0),
        egui::Align2::LEFT_TOP,
        if current {
            tr.current_input
        } else {
            tr.saved_password
        },
        egui::FontId::proportional(10.5),
        DnfLoginApp::c_text3(),
    );
    if multi {
        ui.put(
            egui::Rect::from_center_size(
                egui::pos2(rect.right() - 18.0, rect.center().y),
                egui::vec2(22.0, 22.0),
            ),
            egui::Checkbox::without_text(selected),
        );
    }
    let clicked = response.clicked();
    response.on_hover_text(name);
    (clicked, remove)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn growing_content_and_results_do_not_displace_launch_button() {
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(960.0, 540.0));
        let mut reference = None;
        for rows in [0, 2, 50, 200] {
            let ctx = egui::Context::default();
            let mut button = egui::Rect::NOTHING;
            let mut bottom = 0.0;
            for _ in 0..2 {
                let _ = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(screen),
                        ..Default::default()
                    },
                    |ui| {
                        egui::Window::new("layout_test")
                            .title_bar(false)
                            .fixed_size(egui::vec2(360.0, card_height(screen.height())))
                            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, -22.0))
                            .show(ui.ctx(), |ui| {
                                ui.spacing_mut().item_spacing.y = 4.0;
                                ui.allocate_ui(
                                    egui::vec2(ui.available_width(), HEADER_HEIGHT),
                                    |ui| {
                                        ui.label("Sign in");
                                    },
                                );
                                login_body(
                                    ui,
                                    card_height(screen.height())
                                        - HEADER_HEIGHT
                                        - FOOTER_HEIGHT
                                        - 8.0,
                                    |ui| {
                                        for _ in 0..rows {
                                            ui.label("Saved account or form content");
                                        }
                                    },
                                );
                                ui.add_space(6.0);
                                button = login_button(ui, "Launch selected (2)", true).rect;
                                status_line(
                                    ui,
                                    &format!(
                                        "{}\n{}",
                                        "long error ".repeat(100),
                                        "Account: launch failed\n".repeat(rows.max(1))
                                    ),
                                    true,
                                    Some("Details"),
                                );
                                ui.label("Register / Settings");
                                bottom = ui.min_rect().bottom();
                            });
                    },
                );
            }
            assert!(
                screen.contains_rect(button),
                "launch button must remain visible for {rows} rows"
            );
            assert!(
                bottom < screen.bottom() - 50.0,
                "footer must clear the background switcher"
            );
            if let Some(rect) = reference {
                assert_eq!(button, rect, "content must not move the launch button");
            }
            reference = Some(button);
        }
    }

    #[test]
    fn account_fill_and_batch_checkbox_are_independent() {
        fn draw(ctx: &egui::Context, events: Vec<egui::Event>, selected: &mut bool) -> bool {
            let mut filled = false;
            let _ = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(360.0, 60.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| {
                    filled = account_card(
                        ui,
                        &"long-account-name".repeat(30),
                        false,
                        selected,
                        true,
                        crate::i18n::translations(crate::i18n::Language::English),
                    )
                    .0;
                    assert!(
                        ui.min_rect().width() <= 360.0,
                        "long names must be clipped inside the row"
                    );
                },
            );
            filled
        }

        fn click(x: f32) -> Vec<egui::Event> {
            let pos = egui::pos2(x, 20.0);
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ]
        }

        let ctx = egui::Context::default();
        let mut selected = false;
        draw(&ctx, vec![], &mut selected);
        assert!(
            !draw(&ctx, click(342.0), &mut selected),
            "checking must not fill the form"
        );
        assert!(selected);
        assert!(
            draw(&ctx, click(80.0), &mut selected),
            "clicking the name must fill the form"
        );
        assert!(selected, "filling the form must preserve batch selection");
    }
}
