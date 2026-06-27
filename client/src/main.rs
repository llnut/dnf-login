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

#![cfg_attr(windows, windows_subsystem = "windows")]

mod app;
mod config;
mod game;
mod i18n;
mod network;
mod platform;
mod storage;

use anyhow::Result;
use eframe::egui;

/// Decodes the embedded DNF.ico and returns icon data for the OS window.
fn load_window_icon() -> egui::IconData {
    const ICON_BYTES: &[u8] = include_bytes!("../resources/DNF.ico");
    let dir = ico::IconDir::read(std::io::Cursor::new(ICON_BYTES))
        .expect("Failed to parse embedded DNF.ico");
    let entry = dir
        .entries()
        .iter()
        .filter(|e| e.width() >= 32)
        .max_by_key(|e| e.width())
        .or_else(|| dir.entries().first())
        .expect("DNF.ico contains no icon entries");
    let image = entry.decode().expect("Failed to decode icon entry");
    egui::IconData {
        rgba: image.rgba_data().to_vec(),
        width: image.width(),
        height: image.height(),
    }
}

/// Attach the parent process console on Windows so logs stay visible when launched from a terminal.
#[cfg(windows)]
fn attach_parent_console() {
    use windows::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

fn main() -> Result<()> {
    #[cfg(windows)]
    attach_parent_console();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    tracing::info!("DNF Launcher starting...");

    let icon = load_window_icon();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("DNF Launcher")
            .with_inner_size([960.0, 540.0])
            .with_resizable(false)
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "DNF Launcher",
        native_options,
        Box::new(|cc| Ok(Box::new(app::DnfLoginApp::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run application: {:?}", e))
}
