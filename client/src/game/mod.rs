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

use crate::platform;
use anyhow::Result;

pub struct DnfLauncher;

impl DnfLauncher {
    pub fn launch_with_token(
        token: &str,
        plugins_path: &str,
        inject_enabled: bool,
        server_ip: &str,
    ) -> Result<()> {
        platform::launch_dnf(token, plugins_path, inject_enabled, server_ip)?;
        tracing::info!("DNF launched");
        Ok(())
    }
}
