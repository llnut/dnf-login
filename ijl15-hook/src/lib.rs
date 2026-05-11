//! ijl15.dll proxy, version independent hook loader.
//!
//! On DLL_PROCESS_ATTACH:
//!   1. Hooks gethostbyname for DNS redirect.
//!   2. Loads plugin DLLs from a configurable directory.
//!
//! Environment variables, set by the launcher:
//!   - `DNF_PLUGIN_ENABLED`: "0" to disable plugin loading, defaults to enabled
//!   - `DNF_PLUGIN_PATH`: plugin path, defaults to "plugins"
//!     Supports relative, absolute, and special-character paths.
//!
//! Lookups for "start.dnf.tw" return FAKE_HOSTENT directly.

#![allow(non_snake_case)]
#![allow(clippy::upper_case_acronyms)]

#[cfg(not(target_arch = "x86"))]
compile_error!("This crate targets 32-bit x86 only");

use core::cell::UnsafeCell;
use core::ffi::CStr;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};

use hook_common::{
    BOOL, DWORD, HMODULE, TrampolineSlot, finalize_trampoline_pool, fmt_hex32, install_hook,
    log_line, parse_ipv4, parse_ipv4_octets, resolve,
};
use serde::Deserialize;

const DLL_PROCESS_ATTACH: DWORD = 1;
const AF_INET: i16 = 2;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryW(lpLibFileName: *const u16) -> HMODULE;
    fn GetEnvironmentVariableA(lpName: *const i8, lpBuffer: *mut i8, nSize: DWORD) -> DWORD;
    fn GetEnvironmentVariableW(lpName: *const u16, lpBuffer: *mut u16, nSize: DWORD) -> DWORD;
    fn GetModuleFileNameW(hModule: HMODULE, lpFilename: *mut u16, nSize: DWORD) -> DWORD;
    fn FindFirstFileW(
        lpFileName: *const u16,
        lpFindFileData: *mut Win32FindDataW,
    ) -> *mut core::ffi::c_void;
    fn FindNextFileW(
        hFindFile: *mut core::ffi::c_void,
        lpFindFileData: *mut Win32FindDataW,
    ) -> BOOL;
    fn FindClose(hFindFile: *mut core::ffi::c_void) -> BOOL;
    fn CreateFileA(
        lpFileName: *const i8,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: *mut core::ffi::c_void,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: *mut core::ffi::c_void,
    ) -> *mut core::ffi::c_void;
    fn ReadFile(
        hFile: *mut core::ffi::c_void,
        lpBuffer: *mut core::ffi::c_void,
        nNumberOfBytesToRead: DWORD,
        lpNumberOfBytesRead: *mut DWORD,
        lpOverlapped: *mut core::ffi::c_void,
    ) -> BOOL;
    fn CloseHandle(hObject: *mut core::ffi::c_void) -> BOOL;
    fn GetLastError() -> DWORD;
}

/// WinSock-compatible hostent layout.
#[repr(C)]
struct Hostent {
    h_name: *mut i8,
    h_aliases: *mut *mut i8,
    h_addrtype: i16,
    h_length: i16,
    h_addr_list: *mut *mut i8,
}

// SAFETY: Written once during DllMain which runs
// single-threaded under loader lock. Read-only from all threads thereafter.
unsafe impl Sync for Hostent {}

/// Static storage backing a `hostent` struct.
///
/// Pointer chain: hostent.h_addr_list -> addr_list[0] -> ip[0..4]
#[repr(C)]
struct FakeHostentStorage {
    ip: [u8; 4],
    addr_list: [*mut i8; 2],
    aliases: *mut i8,
    hostname: [u8; 16],
    hostent: Hostent,
}

// SAFETY: Same single-writer guarantee as Hostent. All interior pointers
// target fields within this struct; valid for the process lifetime.
unsafe impl Sync for FakeHostentStorage {}

/// UnsafeCell wrapper that implements Sync.
/// Written once during DllMain, read-only after.
struct SyncCell<T>(UnsafeCell<T>);
// SAFETY: The inner value is written once during DllMain under loader lock,
// single-threaded, and only read thereafter. No concurrent mutation occurs.
unsafe impl<T> Sync for SyncCell<T> {}
impl<T> SyncCell<T> {
    const fn new(val: T) -> Self {
        Self(UnsafeCell::new(val))
    }
    fn get(&self) -> *mut T {
        self.0.get()
    }
}

static FAKE_STORAGE: SyncCell<FakeHostentStorage> = SyncCell::new(FakeHostentStorage {
    ip: [0; 4],
    addr_list: [ptr::null_mut(), ptr::null_mut()],
    aliases: ptr::null_mut(),
    hostname: *b"start.dnf.tw\0\0\0\0",
    hostent: Hostent {
        h_name: ptr::null_mut(),
        h_aliases: ptr::null_mut(),
        h_addrtype: 0,
        h_length: 0,
        h_addr_list: ptr::null_mut(),
    },
});

static FAKE_READY: AtomicBool = AtomicBool::new(false);

type GethostbynameFn = unsafe extern "system" fn(*const i8) -> *mut u8;
static TRAMPOLINE_GETHOSTBYNAME: TrampolineSlot<GethostbynameFn> = TrampolineSlot::new();

unsafe extern "system" fn hook_gethostbyname(name: *const i8) -> *mut u8 {
    let Some(orig) = (unsafe { TRAMPOLINE_GETHOSTBYNAME.get() }) else {
        unsafe { log_line(&[b"[gethostbyname] no-trampoline\n"]) };
        return ptr::null_mut();
    };

    if !name.is_null() {
        let name_bytes = unsafe { CStr::from_ptr(name) }.to_bytes();
        unsafe { log_line(&[b"[gethostbyname] ", name_bytes, b"\n"]) };
        if FAKE_READY.load(Ordering::Acquire) && name_bytes == b"start.dnf.tw" {
            unsafe {
                log_line(&[b"[gethostbyname] short-circuit -> FAKE_HOSTENT\n"]);
                let s = FAKE_STORAGE.get();
                return &raw mut (*s).hostent as *mut u8;
            }
        }
    }

    let ret = unsafe { orig(name) };
    unsafe { log_line(&[b"[gethostbyname] ret=0x", &fmt_hex32(ret as u32), b"\n"]) };
    ret
}

// Plugin loader

const INVALID_HANDLE_VALUE: usize = !0;

#[repr(C)]
struct Win32FindDataW {
    dw_file_attributes: DWORD,
    _ft_creation_time: [DWORD; 2],
    _ft_last_access_time: [DWORD; 2],
    _ft_last_write_time: [DWORD; 2],
    _n_file_size_high: DWORD,
    _n_file_size_low: DWORD,
    _dw_reserved0: DWORD,
    _dw_reserved1: DWORD,
    c_file_name: [u16; 260],
    _c_alternate_file_name: [u16; 14],
}

/// Returns the length in u16 units of a null-terminated UTF-16 string.
fn wcslen(s: &[u16]) -> usize {
    s.iter().position(|&c| c == 0).unwrap_or(s.len())
}

/// Copies `src` UTF-16 units into `dst`, returns number of units written.
/// Does NOT write a null terminator.
fn wcopy(dst: &mut [u16], src: &[u16]) -> usize {
    let n = src.len().min(dst.len());
    dst[..n].copy_from_slice(&src[..n]);
    n
}

/// Returns true if `name` ends with ".dll", case-insensitive.
/// Needed because FindFirstFileW "*.dll" also matches ".dll.bk" etc.
/// due to legacy 8.3 wildcard behavior.
fn ends_with_dll(name: &[u16]) -> bool {
    if name.len() < 4 {
        return false;
    }
    let tail = &name[name.len() - 4..];
    tail[0] == b'.' as u16
        && (tail[1] == b'd' as u16 || tail[1] == b'D' as u16)
        && (tail[2] == b'l' as u16 || tail[2] == b'L' as u16)
        && (tail[3] == b'l' as u16 || tail[3] == b'L' as u16)
}

/// Reads `DNF_PLUGIN_ENABLED` env var. Returns true unless the value is "0".
unsafe fn is_plugin_enabled() -> bool {
    let mut buf = [0i8; 8];
    let len = unsafe {
        GetEnvironmentVariableA(
            c"DNF_PLUGIN_ENABLED".as_ptr(),
            buf.as_mut_ptr(),
            buf.len() as DWORD,
        )
    };
    if len == 0 || len as usize >= buf.len() {
        return true;
    }
    !(len == 1 && buf[0] == b'0' as i8)
}

/// Reads `DNF_PLUGIN_PATH` env var into `out` as UTF-16.
///
/// The value is a file-system path to the plugin directory.
/// Returns the length in u16 units, or 0 if not set.
unsafe fn read_plugin_path_env(out: &mut [u16]) -> usize {
    let name: [u16; 16] = [
        b'D' as u16,
        b'N' as u16,
        b'F' as u16,
        b'_' as u16,
        b'P' as u16,
        b'L' as u16,
        b'U' as u16,
        b'G' as u16,
        b'I' as u16,
        b'N' as u16,
        b'_' as u16,
        b'P' as u16,
        b'A' as u16,
        b'T' as u16,
        b'H' as u16,
        0,
    ];
    let len =
        unsafe { GetEnvironmentVariableW(name.as_ptr(), out.as_mut_ptr(), out.len() as DWORD) };
    if len == 0 || len as usize >= out.len() {
        return 0;
    }
    len as usize
}

/// Builds the plugin directory path as null-terminated UTF-16 in `out`.
///
/// Returns the path length excluding null, or 0 on failure.
/// If `DNF_PLUGIN_PATH` is an absolute path, uses it directly.
/// If relative or default "plugins", resolves relative to DNF.exe's directory.
unsafe fn build_plugin_path(hmodule: HMODULE, out: &mut [u16; 600]) -> usize {
    let mut env_path = [0u16; 520];
    let env_len = unsafe { read_plugin_path_env(&mut env_path) };

    let (dir_ptr, path_len) = if env_len > 0 {
        (&env_path[..], env_len)
    } else {
        let default: [u16; 7] = [
            b'p' as u16,
            b'l' as u16,
            b'u' as u16,
            b'g' as u16,
            b'i' as u16,
            b'n' as u16,
            b's' as u16,
        ];
        env_path[..7].copy_from_slice(&default);
        (&env_path[..], 7usize)
    };

    // Check if path is absolute: drive letter or UNC
    let is_absolute = (path_len >= 3 && dir_ptr[1] == b':' as u16)
        || (path_len >= 2 && dir_ptr[0] == b'\\' as u16 && dir_ptr[1] == b'\\' as u16);

    if is_absolute {
        let n = wcopy(&mut out[..], &dir_ptr[..path_len]);
        let n = if n > 0 && out[n - 1] == b'\\' as u16 {
            n - 1
        } else {
            n
        };
        out[n] = 0;
        return n;
    }

    // Relative path: resolve from the DLL's own directory.
    // GetModuleFileNameW with NULL returns DNF.exe's path.
    // Pass hmodule instead to get the ijl15.dll directory.
    let exe_len = unsafe { GetModuleFileNameW(hmodule, out.as_mut_ptr(), 520) } as usize;
    if exe_len == 0 || exe_len >= 520 {
        return 0;
    }
    let last_sep = out[..exe_len]
        .iter()
        .rposition(|&c| c == b'\\' as u16 || c == b'/' as u16)
        .unwrap_or(0);
    let base = last_sep + 1;

    let n = wcopy(&mut out[base..], &dir_ptr[..path_len]);
    let total = base + n;
    if total >= out.len() {
        return 0;
    }
    out[total] = 0;
    total
}

/// Scans `dir` for *.dll files and loads each one via LoadLibraryW.
unsafe fn load_plugins(hmodule: HMODULE) {
    if !unsafe { is_plugin_enabled() } {
        unsafe { log_line(&[b"[plugins] disabled by DNF_PLUGIN_ENABLED=0\n"]) };
        return;
    }

    let mut path_buf = [0u16; 600];
    let path_len = unsafe { build_plugin_path(hmodule, &mut path_buf) };
    if path_len == 0 {
        unsafe { log_line(&[b"[plugins] cannot resolve plugin path\n"]) };
        return;
    }

    // Log directory path as ASCII, best-effort for non-ASCII chars
    let mut path_ascii = [0u8; 256];
    let ascii_len = path_len.min(path_ascii.len());
    for i in 0..ascii_len {
        path_ascii[i] = if path_buf[i] < 128 {
            path_buf[i] as u8
        } else {
            b'?'
        };
    }
    unsafe { log_line(&[b"[plugins] path=", &path_ascii[..ascii_len], b"\n"]) };

    let mut pattern = [0u16; 620];
    let mut pos = wcopy(&mut pattern, &path_buf[..path_len]);
    pattern[pos] = b'\\' as u16;
    pos += 1;
    pattern[pos] = b'*' as u16;
    pos += 1;
    pattern[pos] = b'.' as u16;
    pos += 1;
    pattern[pos] = b'd' as u16;
    pos += 1;
    pattern[pos] = b'l' as u16;
    pos += 1;
    pattern[pos] = b'l' as u16;
    pos += 1;
    pattern[pos] = 0;

    let mut fd = core::mem::MaybeUninit::<Win32FindDataW>::uninit();
    let hfind = unsafe { FindFirstFileW(pattern.as_ptr(), fd.as_mut_ptr()) };
    if hfind.is_null() || hfind as usize == INVALID_HANDLE_VALUE {
        unsafe { log_line(&[b"[plugins] no DLLs found\n"]) };
        return;
    }

    let mut count = 0u32;
    loop {
        let fd_ref = unsafe { fd.assume_init_ref() };
        let name_len = wcslen(&fd_ref.c_file_name);
        if name_len > 0
            && (fd_ref.dw_file_attributes & 0x10) == 0
            && ends_with_dll(&fd_ref.c_file_name[..name_len])
        {
            let mut full = [0u16; 600];
            let mut p = wcopy(&mut full, &path_buf[..path_len]);
            // Skip if path + backslash + filename + null would overflow
            if p + 1 + name_len >= full.len() {
                if unsafe { FindNextFileW(hfind, fd.as_mut_ptr()) } == 0 {
                    break;
                }
                continue;
            }
            full[p] = b'\\' as u16;
            p += 1;
            p += wcopy(&mut full[p..], &fd_ref.c_file_name[..name_len]);
            full[p] = 0;

            let h = unsafe { LoadLibraryW(full.as_ptr()) };

            // Log filename as ASCII, best-effort
            let mut name_ascii = [0u8; 128];
            let na_len = name_len.min(name_ascii.len());
            for (dst, &src) in name_ascii[..na_len]
                .iter_mut()
                .zip(&fd_ref.c_file_name[..na_len])
            {
                *dst = if src < 128 { src as u8 } else { b'?' };
            }

            if h.is_null() {
                let err = unsafe { GetLastError() };
                unsafe {
                    log_line(&[
                        b"[plugins] FAIL ",
                        &name_ascii[..na_len],
                        b" err=0x",
                        &fmt_hex32(err),
                        b"\n",
                    ])
                };
            } else {
                unsafe {
                    log_line(&[
                        b"[plugins] loaded ",
                        &name_ascii[..na_len],
                        b" at 0x",
                        &fmt_hex32(h as u32),
                        b"\n",
                    ]);
                }
                count += 1;
            }
        }

        if unsafe { FindNextFileW(hfind, fd.as_mut_ptr()) } == 0 {
            break;
        }
    }
    unsafe { FindClose(hfind) };
    unsafe {
        log_line(&[b"[plugins] loaded ", &fmt_hex32(count), b" plugin(s)\n"]);
    }
}

// ijl15.dll export stubs, no-op to satisfy DLL interface contract

#[unsafe(no_mangle)]
pub extern "system" fn ijlInit() -> i32 {
    0
}
#[unsafe(no_mangle)]
pub extern "system" fn ijlFree() -> i32 {
    0
}
#[unsafe(no_mangle)]
pub extern "system" fn ijlRead() -> i32 {
    0
}
#[unsafe(no_mangle)]
pub extern "system" fn ijlWrite() -> i32 {
    0
}
#[unsafe(no_mangle)]
pub extern "system" fn ijlGetLibVersion() -> i32 {
    0
}
#[unsafe(no_mangle)]
pub extern "system" fn ijlErrorStr() -> *const i8 {
    c"".as_ptr()
}

/// Subset of GameSettings.toml that ijl15-hook consumes. Other fields are
/// tolerated as unknown.
#[derive(Deserialize)]
struct IjlConfig {
    game_server_ip: Option<String>,
}

/// Reads `game_server_ip` from `GameSettings.toml` in the working directory.
///
/// # Safety
/// Calls Win32 file I/O functions.
unsafe fn read_ip_from_toml(out: &mut [u8; 64]) -> usize {
    let h = unsafe {
        CreateFileA(
            c"GameSettings.toml".as_ptr(),
            0x8000_0000, // GENERIC_READ
            1,           // FILE_SHARE_READ
            ptr::null_mut(),
            3,    // OPEN_EXISTING
            0x80, // FILE_ATTRIBUTE_NORMAL
            ptr::null_mut(),
        )
    };
    const INVALID_HANDLE_VALUE: usize = !0;
    if h.is_null() || h as usize == INVALID_HANDLE_VALUE {
        return 0;
    }
    let mut buf = [0u8; 4096];
    let mut bytes_read: DWORD = 0;
    let ok = unsafe {
        ReadFile(
            h,
            buf.as_mut_ptr() as *mut core::ffi::c_void,
            buf.len() as DWORD,
            &mut bytes_read,
            ptr::null_mut(),
        )
    };
    unsafe { CloseHandle(h) };
    if ok == 0 || bytes_read == 0 {
        return 0;
    }
    if bytes_read as usize == buf.len() {
        unsafe { log_line(&[b"[read_ip_from_toml] file too large, possible truncation\n"]) };
        return 0;
    }

    let content = match core::str::from_utf8(&buf[..bytes_read as usize]) {
        Ok(s) => s,
        Err(_) => return 0,
    };
    let config: IjlConfig = match toml::from_str(content) {
        Ok(c) => c,
        Err(_) => return 0,
    };
    let ip = match config.game_server_ip {
        Some(ref s) if !s.is_empty() && s.len() < out.len() => s,
        _ => return 0,
    };
    out[..ip.len()].copy_from_slice(ip.as_bytes());
    ip.len()
}

/// Resolves the server IP from the environment variable or TOML config.
/// Returns the IP byte length in `ip_buf`, or 0 if no valid IP is found.
///
/// # Safety
/// Calls Win32 API functions.
unsafe fn resolve_server_ip(ip_buf: &mut [u8; 64]) -> usize {
    let mut env_buf = [0i8; 64];
    let len = unsafe {
        GetEnvironmentVariableA(
            c"GAME_SERVER_IP".as_ptr(),
            env_buf.as_mut_ptr(),
            env_buf.len() as DWORD,
        )
    };
    if len > 0 && (len as usize) < env_buf.len() {
        let ip_bytes = unsafe { CStr::from_ptr(env_buf.as_ptr()) }.to_bytes();
        if parse_ipv4(ip_bytes) {
            ip_buf[..ip_bytes.len()].copy_from_slice(ip_bytes);
            unsafe { log_line(&[b"[on_attach] server_ip=", ip_bytes, b" (env)\n"]) };
            return ip_bytes.len();
        }
        unsafe { log_line(&[b"[on_attach] GAME_SERVER_IP invalid\n"]) };
    }

    // Fall back to GameSettings.toml.
    let toml_len = unsafe { read_ip_from_toml(ip_buf) };
    if toml_len > 0 {
        let ip_slice = &ip_buf[..toml_len];
        if parse_ipv4(ip_slice) {
            unsafe { log_line(&[b"[on_attach] server_ip=", ip_slice, b" (toml)\n"]) };
            return toml_len;
        }
        unsafe { log_line(&[b"[on_attach] game_server_ip invalid (toml)\n"]) };
    }

    0
}

unsafe fn init_fake_hostent(ip_bytes: &[u8]) {
    let octets = parse_ipv4_octets(ip_bytes);
    let s = FAKE_STORAGE.get();
    unsafe {
        (*s).ip = octets;
        (*s).addr_list = [&raw mut (*s).ip as *mut u8 as *mut i8, ptr::null_mut()];
        (*s).hostent = Hostent {
            h_name: &raw mut (*s).hostname as *mut u8 as *mut i8,
            h_aliases: &raw mut (*s).aliases,
            h_addrtype: AF_INET,
            h_length: 4,
            h_addr_list: &raw mut (*s).addr_list as *mut *mut i8,
        };
    }
    FAKE_READY.store(true, Ordering::Release);
}

unsafe fn on_attach(hmodule: HMODULE) {
    unsafe { hook_common::log_open(c"ijl15.log".as_ptr()) };
    unsafe { log_line(&[b"[on_attach] entry\n"]) };

    // 1. Hook gethostbyname for DNS redirect
    let mut ip_buf = [0u8; 64];
    let ip_len = unsafe { resolve_server_ip(&mut ip_buf) };
    if ip_len > 0 {
        unsafe { init_fake_hostent(&ip_buf[..ip_len]) };
        unsafe { log_line(&[b"[on_attach] FAKE_HOSTENT ready\n"]) };

        let gethostbyname_addr = {
            let mut a = unsafe { resolve(b"ws2_32.dll\0", b"gethostbyname\0") };
            if a == 0 {
                a = unsafe { resolve(b"wsock32.dll\0", b"gethostbyname\0") };
            }
            a
        };
        unsafe {
            log_line(&[
                b"[on_attach] gethostbyname=0x",
                &fmt_hex32(gethostbyname_addr as u32),
                b"\n",
            ]);
        }
        if gethostbyname_addr != 0 {
            let ok = unsafe {
                install_hook(
                    gethostbyname_addr,
                    hook_gethostbyname as *const () as usize,
                    TRAMPOLINE_GETHOSTBYNAME.as_raw(),
                )
            };
            let status: &[u8] = if ok { b"ok\n" } else { b"FAILED\n" };
            unsafe { log_line(&[b"[on_attach] hook_gethostbyname=", status]) };
        }
    } else {
        unsafe { log_line(&[b"[on_attach] no valid server IP, DNS hook skipped\n"]) };
    }

    // 2. Load plugins
    unsafe { load_plugins(hmodule) };

    // Freeze the trampoline pool now that no further hooks will be installed.
    unsafe { finalize_trampoline_pool() };

    unsafe { log_line(&[b"[on_attach] done\n"]) };
}

/// # Safety
/// Called by the OS loader.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllMain(
    hmodule: HMODULE,
    reason: DWORD,
    _reserved: *mut core::ffi::c_void,
) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        unsafe { hook_common::DisableThreadLibraryCalls(hmodule) };
        unsafe { on_attach(hmodule) };
    }
    1
}
