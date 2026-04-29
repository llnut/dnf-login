use eframe::egui;

use super::DnfLoginApp;

// UI components
impl DnfLoginApp {
    pub(super) fn text_input<'a>(
        label: &str,
        value: &'a mut String,
        hint: &'a str,
    ) -> impl egui::Widget + 'a {
        let label = label.to_string();
        move |ui: &mut egui::Ui| {
            ui.label(
                egui::RichText::new(&label)
                    .size(13.0)
                    .color(Self::c_text2()),
            );
            ui.add_space(4.0);
            ui.add(
                egui::TextEdit::singleline(value)
                    .desired_width(f32::INFINITY)
                    .min_size(egui::vec2(0.0, 20.0))
                    .margin(egui::vec2(4.0, 2.75))
                    .hint_text(hint)
                    .font(egui::TextStyle::Body),
            )
        }
    }

    pub(super) fn password_input<'a>(
        label: &str,
        value: &'a mut String,
        hint: &'a str,
    ) -> impl egui::Widget + 'a {
        let label = label.to_string();
        move |ui: &mut egui::Ui| {
            ui.label(
                egui::RichText::new(&label)
                    .size(13.0)
                    .color(Self::c_text2()),
            );
            ui.add_space(4.0);
            ui.add(
                egui::TextEdit::singleline(value)
                    .password(true)
                    .desired_width(f32::INFINITY)
                    .min_size(egui::vec2(0.0, 20.0))
                    .margin(egui::vec2(4.0, 2.75))
                    .hint_text(hint)
                    .font(egui::TextStyle::Body),
            )
        }
    }

    pub(super) fn primary_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> bool {
        let h = 44.0;
        let w = ui.available_width();
        let sense = if enabled {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(egui::vec2(w, h), sense);

        if ui.is_rect_visible(rect) {
            let fill = if !enabled {
                Self::c_border_dim()
            } else if response.is_pointer_button_down_on() {
                Self::c_accent_press()
            } else if response.hovered() {
                Self::c_accent_hover()
            } else {
                Self::c_accent()
            };
            ui.painter()
                .rect_filled(rect, egui::CornerRadius::same(8), fill);
            let text_col = if enabled {
                egui::Color32::WHITE
            } else {
                Self::c_text3()
            };
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(14.5),
                text_col,
            );
        }
        response.clicked()
    }

    pub(super) fn primary_button_slim(ui: &mut egui::Ui, label: &str, enabled: bool) -> bool {
        let h = 36.0;
        let w = ui.available_width();
        let sense = if enabled {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(egui::vec2(w, h), sense);

        if ui.is_rect_visible(rect) {
            let fill = if !enabled {
                Self::c_border_dim()
            } else if response.is_pointer_button_down_on() {
                Self::c_accent_press()
            } else if response.hovered() {
                Self::c_accent_hover()
            } else {
                Self::c_accent()
            };
            ui.painter()
                .rect_filled(rect, egui::CornerRadius::same(6), fill);
            let text_col = if enabled {
                egui::Color32::WHITE
            } else {
                Self::c_text3()
            };
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(14.0),
                text_col,
            );
        }
        response.clicked()
    }

    pub(super) fn secondary_button(label: &str) -> egui::Button<'static> {
        egui::Button::new(
            egui::RichText::new(label.to_string())
                .size(13.5)
                .color(Self::c_text2()),
        )
        .fill(egui::Color32::TRANSPARENT)
        .stroke(egui::Stroke::new(1.0, Self::c_border()))
        .corner_radius(egui::CornerRadius::same(6))
        .min_size(egui::vec2(0.0, 36.0))
    }

    pub(super) fn warning_box(ui: &mut egui::Ui, text: &str) {
        egui::Frame::new()
            .fill(Self::c_warn_bg())
            .stroke(egui::Stroke::new(1.0, Self::c_warn_border()))
            .corner_radius(egui::CornerRadius::same(6))
            .inner_margin(egui::vec2(12.0, 8.0))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(text)
                            .size(13.0)
                            .color(Self::c_warn_text()),
                    );
                });
            });
    }

    pub(super) fn status_box(ui: &mut egui::Ui, text: &str, is_error: bool) {
        let (bg, border, color) = if is_error {
            (Self::c_error_bg(), Self::c_error(), Self::c_error())
        } else {
            (Self::c_success_bg(), Self::c_success(), Self::c_success())
        };
        egui::Frame::new()
            .fill(bg)
            .stroke(egui::Stroke::new(1.0, border))
            .corner_radius(egui::CornerRadius::same(6))
            .inner_margin(egui::vec2(14.0, 10.0))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(text).size(13.5).color(color));
                });
            });
    }

    pub(super) fn confirm_dialog(
        ctx: &egui::Context,
        id: &str,
        title: &str,
        message: &str,
        confirm_label: &str,
        cancel_label: &str,
        loading: Option<&str>,
    ) -> Option<bool> {
        let overlay_layer =
            egui::LayerId::new(egui::Order::Middle, egui::Id::new(format!("{id}_overlay")));
        ctx.layer_painter(overlay_layer).rect_filled(
            ctx.viewport_rect(),
            0.0,
            egui::Color32::from_black_alpha(140),
        );

        let glass = egui::Frame::new()
            .fill(Self::c_glass_fill())
            .stroke(egui::Stroke::new(1.0, Self::c_glass_border()))
            .corner_radius(egui::CornerRadius::same(14))
            .inner_margin(egui::vec2(24.0, 20.0));

        let mut action = None;

        egui::Window::new(id)
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .min_width(340.0)
            .max_width(340.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .frame(glass)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new(title)
                            .size(19.0)
                            .strong()
                            .color(Self::c_text()),
                    );
                });
                ui.add_space(10.0);

                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new(message)
                            .size(13.5)
                            .color(Self::c_text2()),
                    );
                });
                ui.add_space(16.0);

                if let Some(loading_text) = loading {
                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(
                                egui::RichText::new(loading_text)
                                    .size(13.0)
                                    .color(Self::c_text2()),
                            );
                        });
                    });
                } else {
                    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if Self::primary_button_slim(ui, confirm_label, true) || enter_pressed {
                        action = Some(true);
                    }
                    ui.add_space(8.0);
                    if ui
                        .add_sized(
                            egui::vec2(ui.available_width(), 36.0),
                            Self::secondary_button(cancel_label),
                        )
                        .clicked()
                    {
                        action = Some(false);
                    }
                }
            });

        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect_shape_with_fill(
        shape: &egui::epaint::ClippedShape,
        fill: egui::Color32,
    ) -> Option<egui::Rect> {
        rect_in_shape_with_fill(&shape.shape, fill)
    }

    fn rect_in_shape_with_fill(shape: &egui::Shape, fill: egui::Color32) -> Option<egui::Rect> {
        match shape {
            egui::Shape::Rect(rect) if rect.fill == fill => Some(rect.rect),
            egui::Shape::Vec(shapes) => shapes
                .iter()
                .find_map(|shape| rect_in_shape_with_fill(shape, fill)),
            _ => None,
        }
    }

    fn contains_dialog_frame(shape: &egui::Shape, screen: egui::Rect) -> bool {
        match shape {
            egui::Shape::Rect(rect) => {
                rect.corner_radius == egui::CornerRadius::same(14)
                    && rect.rect.contains(screen.center())
                    && rect.rect.width() >= 340.0
                    && rect.rect.width() < screen.width()
                    && rect.rect.height() > 100.0
            }
            egui::Shape::Vec(shapes) => shapes
                .iter()
                .any(|shape| contains_dialog_frame(shape, screen)),
            _ => false,
        }
    }

    #[test]
    fn confirm_dialog_overlay_is_painted_below_dialog() {
        let ctx = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0));

        let input = || egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };
        let show_dialog = |ui: &mut egui::Ui| {
            DnfLoginApp::confirm_dialog(
                ui.ctx(),
                "test_close_dialog",
                "Game Already Running",
                "DNF.exe is already running.",
                "Close and continue",
                "Cancel",
                None,
            );
        };

        // The first frame is egui's sizing pass for a newly-created Window.
        let _ = ctx.run_ui(input(), show_dialog);
        let output = ctx.run_ui(input(), show_dialog);

        let overlay_idx = output
            .shapes
            .iter()
            .position(|shape| {
                rect_shape_with_fill(shape, egui::Color32::from_black_alpha(140))
                    .is_some_and(|rect| rect == screen)
            })
            .expect("confirm dialog should paint a full-screen overlay");

        let dialog_idx = output
            .shapes
            .iter()
            .position(|shape| contains_dialog_frame(&shape.shape, screen))
            .expect("confirm dialog should paint its frame");

        assert!(
            overlay_idx < dialog_idx,
            "overlay must be painted before the dialog frame so it cannot cover the dialog"
        );
    }
}
