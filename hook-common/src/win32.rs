//! Win32 API extern declarations and constants used by the other modules.

use crate::{BOOL, DWORD, HMODULE};

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn GetModuleHandleA(lpModuleName: *const i8) -> HMODULE;
    pub fn GetProcAddress(hModule: HMODULE, lpProcName: *const i8) -> *mut core::ffi::c_void;
    pub fn DisableThreadLibraryCalls(hLibModule: HMODULE) -> BOOL;
    pub fn VirtualProtect(
        lpAddress: *mut core::ffi::c_void,
        dwSize: usize,
        flNewProtect: DWORD,
        lpflOldProtect: *mut DWORD,
    ) -> BOOL;
    pub fn VirtualAlloc(
        lpAddress: *mut core::ffi::c_void,
        dwSize: usize,
        flAllocationType: DWORD,
        flProtect: DWORD,
    ) -> *mut core::ffi::c_void;
    pub fn CreateFileA(
        lpFileName: *const i8,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: *mut core::ffi::c_void,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: *mut core::ffi::c_void,
    ) -> *mut core::ffi::c_void;
    pub fn WriteFile(
        hFile: *mut core::ffi::c_void,
        lpBuffer: *const core::ffi::c_void,
        nNumberOfBytesToWrite: DWORD,
        lpNumberOfBytesWritten: *mut DWORD,
        lpOverlapped: *mut core::ffi::c_void,
    ) -> BOOL;
    pub fn FlushFileBuffers(hFile: *mut core::ffi::c_void) -> BOOL;
}

pub const MEM_COMMIT_RESERVE: DWORD = 0x3000;
pub const PAGE_EXECUTE_READWRITE: DWORD = 0x40;
pub const PAGE_EXECUTE_READ: DWORD = 0x20;
