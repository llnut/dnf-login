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

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(not(target_os = "windows"))]
pub fn get_mac_address() -> anyhow::Result<String> {
    anyhow::bail!("This application only supports Windows")
}

#[cfg(not(target_os = "windows"))]
pub fn is_process_running(_process_name: &str) -> anyhow::Result<bool> {
    anyhow::bail!("This application only supports Windows")
}

#[cfg(not(target_os = "windows"))]
pub fn graceful_terminate_process(_process_name: &str) -> anyhow::Result<()> {
    anyhow::bail!("This application only supports Windows")
}

#[cfg(not(target_os = "windows"))]
pub fn launch_dnf(
    _token: &str,
    _plugins_path: &str,
    _inject_enabled: bool,
    _server_ip: &str,
) -> anyhow::Result<()> {
    anyhow::bail!("This application only supports Windows")
}
