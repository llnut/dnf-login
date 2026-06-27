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

//! 5-byte JMP inline hook installation with a pooled trampoline allocator.
//!
//! Trampolines are 32 bytes each, packed into 4 KiB pages allocated on
//! demand via VirtualAlloc. The current page is held
//! PAGE_EXECUTE_READWRITE while it accepts writes. Once it fills up or
//! the caller invokes [`finalize_trampoline_pool`], the page is
//! downgraded to PAGE_EXECUTE_READ.

use core::marker::PhantomData;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::DWORD;
use crate::decoder::{decode_hook_target, trampoline_copy_size};
use crate::log::{fmt_hex8, fmt_hex32, log_line};
use crate::win32::{
    MEM_COMMIT_RESERVE, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, VirtualAlloc, VirtualProtect,
};

const TRAMPOLINE_SIZE: usize = 32;
const POOL_PAGE_SIZE: usize = 4096;

// Current writable page and the offset of the next free slot.
static POOL_PAGE: AtomicUsize = AtomicUsize::new(0);
static POOL_OFFSET: AtomicUsize = AtomicUsize::new(0);

/// Downgrades `page` from PAGE_EXECUTE_READWRITE to PAGE_EXECUTE_READ.
/// Failure is logged; the page stays writable.
unsafe fn freeze_page(page: usize) {
    let mut old: DWORD = 0;
    let ok = unsafe { VirtualProtect(page as *mut _, POOL_PAGE_SIZE, PAGE_EXECUTE_READ, &mut old) };
    if ok == 0 {
        unsafe {
            log_line(&[
                b"[trampoline] freeze FAILED page=0x",
                &fmt_hex32(page as u32),
                b"\n",
            ]);
        }
    }
}

/// Reserves a 32-byte trampoline slot. A fresh RWX page is allocated
/// when the current page is full or absent; the previous page is
/// frozen to RX first, so at most one page is writable at a time.
/// Returns 0 on VirtualAlloc failure.
unsafe fn alloc_trampoline_slot() -> usize {
    loop {
        let page = POOL_PAGE.load(Ordering::Acquire);
        let offset = POOL_OFFSET.load(Ordering::Acquire);
        if page != 0 && offset + TRAMPOLINE_SIZE <= POOL_PAGE_SIZE {
            match POOL_OFFSET.compare_exchange(
                offset,
                offset + TRAMPOLINE_SIZE,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return page + offset,
                Err(_) => continue,
            }
        }
        // Page absent or exhausted. Freeze the old one and commit a new
        // one. The freeze is best-effort; failure is logged.
        if page != 0 {
            unsafe { freeze_page(page) };
        }
        let new_page = unsafe {
            VirtualAlloc(
                ptr::null_mut(),
                POOL_PAGE_SIZE,
                MEM_COMMIT_RESERVE,
                PAGE_EXECUTE_READWRITE,
            )
        };
        if new_page.is_null() {
            POOL_PAGE.store(0, Ordering::Release);
            POOL_OFFSET.store(0, Ordering::Release);
            return 0;
        }
        let base = new_page as usize;
        POOL_PAGE.store(base, Ordering::Release);
        POOL_OFFSET.store(TRAMPOLINE_SIZE, Ordering::Release);
        return base;
    }
}

/// Freezes the current pool page to PAGE_EXECUTE_READ and detaches it from
/// the allocator. Call once after all hook installation is finished. A
/// later [`install_hook`] will start a fresh page.
///
/// # Safety
/// Must be called on the same thread that drove `install_hook` and only
/// after every hook has been installed. Concurrent hook installation is
/// not synchronised against this call.
pub unsafe fn finalize_trampoline_pool() {
    let page = POOL_PAGE.load(Ordering::Acquire);
    if page == 0 {
        return;
    }
    unsafe { freeze_page(page) };
    POOL_PAGE.store(0, Ordering::Release);
    POOL_OFFSET.store(0, Ordering::Release);
}

/// Atomic slot holding a typed trampoline address.
///
/// Written once with Release ordering during installation, read with
/// Acquire ordering from the hook path.
pub struct TrampolineSlot<F: Copy>(AtomicUsize, PhantomData<*const F>);

// SAFETY: `AtomicUsize` provides atomic access. `PhantomData<*const F>` is
// only a marker; the slot is written once before any hook can observe it.
unsafe impl<F: Copy> Sync for TrampolineSlot<F> {}

impl<F: Copy> Default for TrampolineSlot<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: Copy> TrampolineSlot<F> {
    pub const fn new() -> Self {
        Self(AtomicUsize::new(0), PhantomData)
    }

    pub fn as_raw(&self) -> &AtomicUsize {
        &self.0
    }

    /// Returns the stored function pointer, or `None` if the slot is
    /// still uninitialised.
    ///
    /// # Safety
    /// The value stored via `as_raw().store()` must be a valid, callable
    /// function pointer of type `F`. `size_of::<F>()` must equal
    /// `size_of::<usize>()`.
    pub unsafe fn get(&self) -> Option<F> {
        const { assert!(core::mem::size_of::<F>() == core::mem::size_of::<usize>()) };
        let v = self.0.load(Ordering::Acquire);
        if v == 0 {
            return None;
        }
        Some(unsafe { core::mem::transmute_copy(&v) })
    }
}

/// Builds a trampoline that returns control to `target + n` where `n`
/// is the bytes copied from the saved prologue. A prologue that is
/// already a `JMP rel32` or `PUSH imm32 ; RET` stub is chained directly
/// to its target instead, avoiding double indirection.
unsafe fn alloc_trampoline(target: usize, saved: [u8; 8]) -> usize {
    let mem = unsafe { alloc_trampoline_slot() };
    if mem == 0 {
        return 0;
    }
    let p = mem as *mut u8;

    let mut hex = [b'0'; 16];
    for (i, &b) in saved.iter().enumerate() {
        let [hi, lo] = fmt_hex8(b);
        hex[i * 2] = hi;
        hex[i * 2 + 1] = lo;
    }
    unsafe { log_line(&[b"[trampoline] bytes=", &hex, b"\n"]) };

    if let Some(dest) = decode_hook_target(target, &saved) {
        let new_rel = (dest as i32).wrapping_sub(mem as i32 + 5);
        unsafe {
            *p = 0xE9;
            ptr::write_unaligned(p.add(1) as *mut i32, new_rel);
        }
        unsafe { log_line(&[b"[trampoline] chain->0x", &fmt_hex32(dest as u32), b"\n"]) };
    } else {
        let n = trampoline_copy_size(&saved);
        unsafe { ptr::copy_nonoverlapping(saved.as_ptr(), p, n) };
        let rel = (target as i32 + n as i32).wrapping_sub(mem as i32 + n as i32 + 5);
        unsafe {
            *p.add(n) = 0xE9;
            ptr::write_unaligned(p.add(n + 1) as *mut i32, rel);
        }
        unsafe { log_line(&[b"[trampoline] copy+jmp n=0x", &fmt_hex8(n as u8), b"\n"]) };
    }
    mem
}

/// Patches `target` with a 5-byte JMP to `hook` and publishes the
/// trampoline through `trampoline_store`. The trampoline is stored with
/// Release ordering before the JMP is written, so any thread reaching
/// the hook observes a valid trampoline.
///
/// # Safety
/// - `target` must be a valid, executable address whose first 8 bytes
///   are readable; they are copied into the trampoline.
/// - No other thread may execute the first 5 bytes of `target` while
///   the hook is being installed. DllMain under loader lock satisfies
///   this.
/// - Concurrent `install_hook` calls must be serialised. The slot bump
///   is CAS-safe, but the new-page path is not, so racing callers may
///   leak a fresh VirtualAlloc page.
pub unsafe fn install_hook(target: usize, hook: usize, trampoline_store: &AtomicUsize) -> bool {
    let target_ptr = target as *mut u8;
    let saved = unsafe { *(target as *const [u8; 8]) };

    let trampoline = unsafe { alloc_trampoline(target, saved) };
    if trampoline == 0 {
        return false;
    }
    trampoline_store.store(trampoline, Ordering::Release);

    let mut old_prot: DWORD = 0;
    if unsafe {
        VirtualProtect(
            target_ptr as *mut _,
            5,
            PAGE_EXECUTE_READWRITE,
            &mut old_prot,
        )
    } == 0
    {
        return false;
    }
    let rel = hook.wrapping_sub(target + 5) as i32;
    unsafe {
        *target_ptr = 0xE9;
        ptr::write_unaligned(target_ptr.add(1) as *mut i32, rel);
        VirtualProtect(target_ptr as *mut _, 5, old_prot, &mut old_prot);
    }
    true
}
