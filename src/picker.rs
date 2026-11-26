//! Mode "Pick": klik kiri berikutnya di mana pun di layar menjadi posisi klik.
//! Memakai hook mouse low-level hanya selama mode aktif. Klik itu ditelan (tidak sampai ke aplikasi lain),
//! klik kanan membatalkan, dan input sintetis (dari klik otomatis) diabaikan.
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicI32, AtomicIsize, AtomicU32, Ordering};

use crate::win32::*;

/// Posisi dipilih: wParam = x, lParam = y (piksel fisik).
pub const WM_PICKED: u32 = WM_APP + 1;
pub const WM_PICK_CANCEL: u32 = WM_APP + 2;

static TARGET: AtomicIsize = AtomicIsize::new(0);
static CANCEL_BTN: AtomicIsize = AtomicIsize::new(0);
static HOOK: AtomicIsize = AtomicIsize::new(0);
/// 0 = tidak ada, 1 = telan WM_LBUTTONUP berikutnya, 2 = telan WM_RBUTTONUP berikutnya.
static SWALLOW: AtomicU32 = AtomicU32::new(0);
static X: AtomicI32 = AtomicI32::new(0);
static Y: AtomicI32 = AtomicI32::new(0);

pub unsafe fn start(target: HWND, cancel_btn: HWND) -> bool {
    if HOOK.load(Ordering::Relaxed) != 0 {
        return true;
    }
    TARGET.store(target as isize, Ordering::Relaxed);
    CANCEL_BTN.store(cancel_btn as isize, Ordering::Relaxed);
    SWALLOW.store(0, Ordering::Relaxed);
    let h = SetWindowsHookExW(WH_MOUSE_LL, hook, GetModuleHandleW(null()), 0);
    HOOK.store(h as isize, Ordering::Relaxed);
    !h.is_null()
}

pub unsafe fn stop() {
    let h = HOOK.swap(0, Ordering::Relaxed);
    CANCEL_BTN.store(0, Ordering::Relaxed);
    SWALLOW.store(0, Ordering::Relaxed);
    if h != 0 {
        UnhookWindowsHookEx(h as HHOOK);
    }
}

unsafe extern "system" fn hook(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if code >= 0 {
        let m = &*(l as *const MSLLHOOKSTRUCT);
        if m.flags & LLMHF_INJECTED == 0 {
            let target = TARGET.load(Ordering::Relaxed) as HWND;
            let cancel = CANCEL_BTN.load(Ordering::Relaxed) as HWND;
            match w as u32 {
                WM_LBUTTONDOWN => {
                    // Biarkan klik pada tombol Cancel lewat ke jendela ClickLoop agar bisa membatalkan picking.
                    if !cancel.is_null() && WindowFromPoint(m.pt) == cancel {
                        return CallNextHookEx(null_mut(), code, w, l);
                    }
                    X.store(m.pt.x, Ordering::Relaxed);
                    Y.store(m.pt.y, Ordering::Relaxed);
                    SWALLOW.store(1, Ordering::Relaxed);
                    return 1;
                }
                // Posisi dikirim saat tombol dilepas, agar pasangan down/up sama-sama ditelan.
                WM_LBUTTONUP if SWALLOW.load(Ordering::Relaxed) == 1 => {
                    SWALLOW.store(0, Ordering::Relaxed);
                    let x = X.load(Ordering::Relaxed);
                    let y = Y.load(Ordering::Relaxed);
                    PostMessageW(target, WM_PICKED, x as isize as usize, y as isize);
                    return 1;
                }
                WM_RBUTTONDOWN => {
                    SWALLOW.store(2, Ordering::Relaxed);
                    return 1;
                }
                WM_RBUTTONUP if SWALLOW.load(Ordering::Relaxed) == 2 => {
                    SWALLOW.store(0, Ordering::Relaxed);
                    PostMessageW(target, WM_PICK_CANCEL, 0, 0);
                    return 1;
                }
                _ => {}
            }
        }
    }
    CallNextHookEx(null_mut(), code, w, l)
}
