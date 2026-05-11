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
