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

use super::DnfLoginApp;

// Color palette
impl DnfLoginApp {
    pub(super) fn c_bg() -> egui::Color32 {
        egui::Color32::from_rgb(10, 10, 10)
    }
    pub(super) fn c_card() -> egui::Color32 {
        egui::Color32::from_rgb(22, 22, 24)
    }
    pub(super) fn c_input_bg() -> egui::Color32 {
        egui::Color32::from_rgb(14, 14, 16)
    }
    pub(super) fn c_border() -> egui::Color32 {
        egui::Color32::from_rgb(52, 52, 58)
    }
    pub(super) fn c_border_dim() -> egui::Color32 {
        egui::Color32::from_rgb(36, 36, 42)
    }
    pub(super) fn c_accent() -> egui::Color32 {
        egui::Color32::from_rgb(59, 130, 246)
    }
    pub(super) fn c_accent_hover() -> egui::Color32 {
        egui::Color32::from_rgb(96, 165, 250)
    }
    pub(super) fn c_accent_press() -> egui::Color32 {
        egui::Color32::from_rgb(37, 99, 235)
    }
    pub(super) fn c_accent_faint() -> egui::Color32 {
        egui::Color32::from_rgb(10, 28, 65)
    }
    pub(super) fn c_text() -> egui::Color32 {
        egui::Color32::from_rgb(238, 240, 250)
    }
    pub(super) fn c_text2() -> egui::Color32 {
        egui::Color32::from_rgb(205, 212, 232)
    }
    pub(super) fn c_text3() -> egui::Color32 {
        egui::Color32::from_rgb(155, 166, 192)
    }
    pub(super) fn c_success() -> egui::Color32 {
        egui::Color32::from_rgb(100, 170, 250)
    }
    pub(super) fn c_error() -> egui::Color32 {
        egui::Color32::from_rgb(220, 110, 140)
    }
    pub(super) fn c_success_bg() -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(6, 18, 45, 215)
    }
    pub(super) fn c_error_bg() -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(38, 8, 20, 215)
    }
    pub(super) fn c_warn_bg() -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(20, 12, 48, 220)
    }
    pub(super) fn c_warn_border() -> egui::Color32 {
        egui::Color32::from_rgb(100, 70, 180)
    }
    pub(super) fn c_warn_text() -> egui::Color32 {
        egui::Color32::from_rgb(180, 155, 235)
    }
    pub(super) fn c_glass_fill() -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(6, 9, 22, 218)
    }
    pub(super) fn c_glass_border() -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(18, 28, 70, 55)
    }
    pub(super) fn c_overlay() -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(0, 0, 6, 75)
    }
    pub(super) fn c_thumb_active() -> egui::Color32 {
        egui::Color32::from_rgb(96, 165, 250)
    }
    pub(super) fn c_thumb_inactive() -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(30, 40, 80, 90)
    }
    pub(super) fn c_thumb_hover() -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(60, 90, 160, 120)
    }
}
