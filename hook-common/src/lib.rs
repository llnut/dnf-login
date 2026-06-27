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

//! Shared utilities for x86 hook DLLs.

#![no_std]
#![allow(non_snake_case)]

pub type BOOL = i32;
pub type DWORD = u32;
pub type HMODULE = *mut core::ffi::c_void;

mod decoder;
mod ipv4;

#[cfg(target_os = "windows")]
mod inline_hook;
#[cfg(target_os = "windows")]
mod log;
#[cfg(target_os = "windows")]
mod resolve;
#[cfg(target_os = "windows")]
mod win32;

pub use decoder::{decode_hook_target, trampoline_copy_size, x86_instr_len};
pub use ipv4::{parse_ipv4, parse_ipv4_octets};

#[cfg(target_os = "windows")]
pub use inline_hook::{TrampolineSlot, finalize_trampoline_pool, install_hook};
#[cfg(target_os = "windows")]
pub use log::{fmt_hex8, fmt_hex32, log_flush, log_line, log_open};
#[cfg(target_os = "windows")]
pub use resolve::resolve;
#[cfg(target_os = "windows")]
pub use win32::{DisableThreadLibraryCalls, GetModuleHandleA, GetProcAddress};
