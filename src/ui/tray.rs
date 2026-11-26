//! Ikon di area notifikasi dan minimize ke tray.
use crate::shell::{self, CMD_EXIT, CMD_SHOW, CMD_TOGGLE};
use crate::win32::*;

use super::actions::toggle_run;
use super::window::{window_alive, Ui, UiCell};
use super::APP;

/// Panjang maksimum tooltip (Windows menyimpan 128 karakter termasuk akhir string).
const TIP_MAX: usize = 127;

/// Teks tooltip: nama aplikasi dan status, dipotong agar muat di area notifikasi.
fn tip_of(ui: &Ui) -> String {
    let full = format!("{} \u{B7} {}", APP, ui.status);
    if full.chars().count() <= TIP_MAX {
        full
    } else {
        full.chars()
            .take(TIP_MAX - 1)
            .chain(std::iter::once('\u{2026}'))
            .collect()
    }
}

/// Menyembunyikan jendela ke area notifikasi. Bila ikon gagal dibuat, jendela diminimize biasa.
pub(super) unsafe fn to_tray(h: HWND, ui: &UiCell) {
    let (icon, tip) = {
        let u = ui.borrow();
        (u.tray_icon, tip_of(&u))
    };
    if !shell::tray_add(h, icon, &tip) {
        // Lewat DefWindowProcW agar tidak masuk lagi ke penanganan SC_MINIMIZE di wnd_proc.
        DefWindowProcW(h, WM_SYSCOMMAND, SC_MINIMIZE, 0);
        return;
    }
    ui.borrow_mut().in_tray = true;
    ShowWindow(h, SW_HIDE);
    K32EmptyWorkingSet(GetCurrentProcess());
}

/// Menampilkan kembali jendela dari area notifikasi atau keadaan minimized.
pub(super) unsafe fn restore(h: HWND, ui: &UiCell) {
    let was_tray = {
        let mut u = ui.borrow_mut();
        let v = u.in_tray;
        u.in_tray = false;
        v
    };
    if was_tray {
        shell::tray_remove(h);
    }
    if IsIconic(h) != 0 || IsWindowVisible(h) == 0 {
        ShowWindow(h, SW_RESTORE);
    } else {
        ShowWindow(h, SW_SHOW);
    }
    SetForegroundWindow(h);
}

/// Mendaftarkan ulang ikon tray jika Explorer/taskbar di-restart saat aplikasi sedang di tray.
pub(super) unsafe fn readd_tray(h: HWND, ui: &UiCell) {
    let info = {
        let u = ui.borrow();
        u.in_tray.then(|| (u.tray_icon, tip_of(&u)))
    };
    if let Some((icon, tip)) = info {
        shell::tray_add(h, icon, &tip);
    }
}

/// Pesan dari ikon area notifikasi: klik kiri menampilkan jendela, klik kanan membuka menu.
pub(super) unsafe fn on_tray_message(h: HWND, ui: &UiCell, l: LPARAM) {
    match (l as usize & 0xFFFF) as u32 {
        WM_LBUTTONUP | WM_LBUTTONDBLCLK => restore(h, ui),
        WM_RBUTTONUP => {
            let running = ui.borrow().runner.is_running();
            // Lepaskan borrow sebelum TrackPopupMenu karena menu menjalankan modal message loop.
            let cmd = shell::tray_menu(h, running);
            if !window_alive(h) {
                return;
            }
            match cmd {
                CMD_SHOW => restore(h, ui),
                CMD_TOGGLE => toggle_run(h, ui),
                CMD_EXIT => {
                    PostMessageW(h, WM_CLOSE, 0, 0);
                }
                _ => {}
            }
        }
        _ => {}
    }
}

/// Menyegarkan tooltip ikon bila jendela sedang di area notifikasi.
pub(super) unsafe fn sync_tip(h: HWND, ui: &UiCell) {
    let info = {
        let u = ui.borrow();
        u.in_tray.then(|| (u.tray_icon, tip_of(&u)))
    };
    if let Some((icon, tip)) = info {
        shell::tray_tip(h, icon, &tip);
    }
}
