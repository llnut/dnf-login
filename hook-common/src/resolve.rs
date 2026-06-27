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

//! Module / symbol resolution helpers.

use crate::win32::{GetModuleHandleA, GetProcAddress};

/// Returns the export address of `func` from `dll`, or 0 if not found.
///
/// # Safety
/// `dll` and `func` must be null-terminated byte strings.
pub unsafe fn resolve(dll: &[u8], func: &[u8]) -> usize {
    let hmod = unsafe { GetModuleHandleA(dll.as_ptr() as *const i8) };
    if hmod.is_null() {
        return 0;
    }
    let addr = unsafe { GetProcAddress(hmod, func.as_ptr() as *const i8) };
    addr as usize
}
