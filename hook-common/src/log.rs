//! Process-global log file with thread-safe single-write line emission.

use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::DWORD;
use crate::win32::{CreateFileA, FlushFileBuffers, WriteFile};

static LOG_HANDLE: AtomicUsize = AtomicUsize::new(0);
const LOG_FAILED: usize = usize::MAX;

const fn nibble_to_hex(n: u8) -> u8 {
    if n < 10 { b'0' + n } else { b'a' + n - 10 }
}

pub fn fmt_hex32(v: u32) -> [u8; 8] {
    let mut out = [b'0'; 8];
    let mut x = v;
    for i in (0..8).rev() {
        out[i] = nibble_to_hex((x & 0xF) as u8);
        x >>= 4;
    }
    out
}

pub fn fmt_hex8(v: u8) -> [u8; 2] {
    [nibble_to_hex(v >> 4), nibble_to_hex(v & 0xF)]
}

/// Opens a log file for writing. Subsequent calls are no-ops.
///
/// # Safety
/// `filename` must be a valid null-terminated C string pointer.
pub unsafe fn log_open(filename: *const i8) {
    if LOG_HANDLE
        .compare_exchange(0, LOG_FAILED, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    let h = unsafe {
        CreateFileA(
            filename,
            0x4000_0000, // GENERIC_WRITE
            1,           // FILE_SHARE_READ
            ptr::null_mut(),
            2,    // CREATE_ALWAYS
            0x80, // FILE_ATTRIBUTE_NORMAL
            ptr::null_mut(),
        )
    };
    let val = if h.is_null() || h as usize == usize::MAX {
        LOG_FAILED
    } else {
        h as usize
    };
    LOG_HANDLE.store(val, Ordering::Release);
}

/// Emits one log line atomically. `parts` are concatenated into a stack
/// buffer and flushed with a single WriteFile call, so concurrent
/// threads cannot interleave. Output longer than 256 bytes is silently
/// truncated.
///
/// # Safety
/// Must be called after `log_open`. Safe from any thread.
pub unsafe fn log_line(parts: &[&[u8]]) {
    let h = LOG_HANDLE.load(Ordering::Acquire) as *mut core::ffi::c_void;
    if h.is_null() || h as usize == LOG_FAILED {
        return;
    }
    let mut buf = [0u8; 256];
    let mut pos = 0usize;
    for &part in parts {
        let n = part.len().min(buf.len() - pos);
        buf[pos..pos + n].copy_from_slice(&part[..n]);
        pos += n;
        if pos == buf.len() {
            break;
        }
    }
    let mut written: DWORD = 0;
    unsafe {
        WriteFile(
            h,
            buf.as_ptr().cast(),
            pos as DWORD,
            &mut written,
            ptr::null_mut(),
        );
    }
}

/// Forces any buffered log output to disk. Called from VEH so the last
/// few lines survive a fatal exception that kills the process before the
/// OS would otherwise flush kernel buffers.
///
/// # Safety
/// Must be called after `log_open`. Safe from any thread.
pub unsafe fn log_flush() {
    let h = LOG_HANDLE.load(Ordering::Acquire) as *mut core::ffi::c_void;
    if h.is_null() || h as usize == LOG_FAILED {
        return;
    }
    unsafe {
        FlushFileBuffers(h);
    }
}

/// Emits a log line at most once over the lifetime of `$flag`.
#[macro_export]
macro_rules! log_once {
    ($flag:expr, $($part:expr),+ $(,)?) => {
        if !$flag.swap(true, ::core::sync::atomic::Ordering::Relaxed) {
            unsafe { $crate::log_line(&[$($part),+]) };
        }
    };
}
