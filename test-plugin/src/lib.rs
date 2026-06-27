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

//! Injection test DLL.
//!
//! Writes `inject_test.log` to the plugins directory on attach, recording
//! basic process information to confirm that injection succeeded.

#![allow(non_snake_case)]
#![allow(clippy::upper_case_acronyms)]

use std::path::Path;

type HANDLE = *mut core::ffi::c_void;
type HMODULE = *mut core::ffi::c_void;
type BOOL = i32;
type DWORD = u32;

const DLL_PROCESS_ATTACH: DWORD = 1;
const TH32CS_SNAPMODULE: DWORD = 0x0000_0008;
const TH32CS_SNAPMODULE32: DWORD = 0x0000_0010;
const MAX_PATH: usize = 260;

// Field order must match the MODULEENTRY32W layout in the Windows SDK.
#[repr(C)]
struct MODULEENTRY32W {
    dw_size: DWORD,
    th32_module_id: DWORD,
    th32_process_id: DWORD,
    glbl_cnt_usage: DWORD,
    proc_cnt_usage: DWORD,
    mod_base_addr: *mut u8, // 4 bytes on 32-bit
    mod_base_size: DWORD,
    h_module: HMODULE, // 4 bytes on 32-bit
    sz_module: [u16; 256],
    sz_exe_path: [u16; MAX_PATH],
}

impl Default for MODULEENTRY32W {
    fn default() -> Self {
        // SAFETY: all-zero is a valid initial state; dw_size is corrected below.
        let mut s: Self = unsafe { core::mem::zeroed() };
        s.dw_size = core::mem::size_of::<Self>() as DWORD;
        s
    }
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcessId() -> DWORD;
    fn GetCurrentThreadId() -> DWORD;
    fn GetModuleFileNameW(hModule: HMODULE, lpFilename: *mut u16, nSize: DWORD) -> DWORD;
    fn CreateToolhelp32Snapshot(dwFlags: DWORD, th32ProcessID: DWORD) -> HANDLE;
    fn Module32FirstW(hSnapshot: HANDLE, lpme: *mut MODULEENTRY32W) -> BOOL;
    fn Module32NextW(hSnapshot: HANDLE, lpme: *mut MODULEENTRY32W) -> BOOL;
    fn CloseHandle(hObject: HANDLE) -> BOOL;
    fn DisableThreadLibraryCalls(hLibModule: HMODULE) -> BOOL;
}

fn wstr(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

fn module_path(hmodule: HMODULE) -> String {
    let mut buf = [0u16; MAX_PATH];
    let len = unsafe { GetModuleFileNameW(hmodule, buf.as_mut_ptr(), MAX_PATH as DWORD) };
    wstr(&buf[..len as usize])
}

fn collect_log(hmodule: HMODULE) -> String {
    let mut out = String::with_capacity(4096);

    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let pid = unsafe { GetCurrentProcessId() };
    let tid = unsafe { GetCurrentThreadId() };
    let dll_path = module_path(hmodule);
    let exe_path = module_path(core::ptr::null_mut()); // null → current executable

    out.push_str(&format!("pid={pid} tid={tid} t={t}\n"));
    out.push_str(&format!("dll={dll_path} base=0x{:08X}\n", hmodule as usize));
    out.push_str(&format!("exe={exe_path}\n\n"));

    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid);
        if !snap.is_null() && snap as isize != -1 {
            let mut me = MODULEENTRY32W::default();
            if Module32FirstW(snap, &mut me) != 0 {
                loop {
                    let name = wstr(&me.sz_module);
                    out.push_str(&format!(
                        "0x{:08X} {}K {}\n",
                        me.mod_base_addr as usize,
                        me.mod_base_size / 1024,
                        name,
                    ));
                    if Module32NextW(snap, &mut me) == 0 {
                        break;
                    }
                }
            }
            CloseHandle(snap);
        }
    }

    out
}

fn on_attach(hmodule: HMODULE) {
    let log = collect_log(hmodule);

    let dll_path = module_path(hmodule);
    let log_path = Path::new(&dll_path)
        .parent()
        .map(|p| p.join("inject_test.log"))
        .unwrap_or_else(|| Path::new("inject_test.log").to_path_buf());

    let _ = std::fs::write(&log_path, log.as_bytes());
}

/// DLL entry point called by the OS loader.
///
/// # Safety
///
/// Called under the loader lock; re-entrancy is not expected at
/// `DLL_PROCESS_ATTACH`. OS loader guarantees that `hmodule` is
/// a valid module handle for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllMain(
    hmodule: HMODULE,
    reason: DWORD,
    _reserved: *mut core::ffi::c_void,
) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        unsafe { DisableThreadLibraryCalls(hmodule) };
        // Prevent a panic from unwinding across the FFI boundary.
        let _ = std::panic::catch_unwind(|| on_attach(hmodule));
    }
    1
}
