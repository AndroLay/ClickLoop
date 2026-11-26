//! Jendela utama ClickLoop. Berkas ini hanya berisi titik masuk, kelas jendela, dan loop pesan;
//! kontrol, tampilan, dan aksi berada di submodul `src/ui/`.
mod actions;
mod create;
mod draw;
mod events;
mod layout;
mod settings;
mod tray;
mod window;

use std::cell::RefCell;
use std::ffi::c_void;
use std::ptr::{null, null_mut};
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::config::Settings;
use crate::win32::*;

use actions::toggle_run;
use create::build;
use events::{focus_changed, wnd_proc};
use layout::{H, W};
use window::{ui_of, Ui};

/// Nama kelas jendela, dipakai untuk menemukan instans yang sudah berjalan.
const CLASS: &str = "ClickLoopWindow";
/// Nama aplikasi di judul jendela, tooltip tray, dan teks lain.
pub const APP: &str = "ClickLoop";
/// Pesan khusus dari instans kedua untuk memunculkan jendela (termasuk dari tray).
pub const WM_RESTORE_APP: u32 = WM_APP + 5;
/// Pesan Windows saat Explorer/taskbar di-restart (didaftarkan saat startup).
static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);
/// Ukuran tetap: tanpa maximize dan tanpa resize, tetapi bisa diminimize.
const WINDOW_STYLE: u32 = WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_CLIPCHILDREN;

pub(super) fn taskbar_created_msg() -> u32 {
    TASKBAR_CREATED.load(Ordering::Relaxed)
}

/// Ikon dari sumber daya EXE (ID 1) dalam ukuran `size` piksel.
unsafe fn load_icon(size: i32) -> HICON {
    // MAKEINTRESOURCEW(1): nilai integer 1 yang dikemas sebagai pointer resource Win32.
    LoadImageW(
        GetModuleHandleW(null()),
        std::ptr::without_provenance::<u16>(1),
        IMAGE_ICON,
        size,
        size,
        0,
    )
}

/// Jika ClickLoop sudah berjalan: kirim pesan untuk memunculkan jendelanya (termasuk bila di tray).
pub unsafe fn focus_existing() {
    let class = wide(CLASS);
    let h = FindWindowW(class.as_ptr(), null());
    if !h.is_null() {
        PostMessageW(h, WM_RESTORE_APP, 0, 0);
        SetForegroundWindow(h);
    }
}

/// Membuka jendela utama. `start`: mulai klik segera setelah jendela tampil (flag `--start`).
/// Mengembalikan `false` bila kelas jendela atau jendela utama tidak bisa dibuat.
pub fn run(s: Settings, start: bool) -> bool {
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_PER_MONITOR_V2);
        let tb = wide("TaskbarCreated");
        TASKBAR_CREATED.store(RegisterWindowMessageW(tb.as_ptr()), Ordering::Relaxed);
        let icc = INITCOMMONCONTROLSEX {
            size: 8,
            icc: ICC_HOTKEY_CLASS,
        };
        InitCommonControlsEx(&icc);
        let inst = GetModuleHandleW(null());
        let class = wide(CLASS);
        let wc = WNDCLASSEXW {
            cb_size: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfn_wnd_proc: Some(wnd_proc),
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance: inst,
            h_icon: load_icon(GetSystemMetrics(SM_CXICON)),
            h_cursor: LoadCursorW(null_mut(), IDC_ARROW as *const u16),
            hbr_background: null_mut(),
            lpsz_menu_name: null(),
            lpsz_class_name: class.as_ptr(),
            h_icon_sm: load_icon(GetSystemMetrics(SM_CXSMICON)),
        };
        if RegisterClassExW(&wc) == 0 {
            return false;
        }
        let mut rc = RECT {
            left: 0,
            top: 0,
            right: sc(W),
            bottom: sc(H),
        };
        adjust_rect(&mut rc, WINDOW_STYLE, dpi());
        // Posisi terakhir dipakai hanya bila masih ada monitor di sana.
        let (x, y) = s
            .win_pos
            .filter(|&(x, y)| {
                !MonitorFromPoint(
                    POINT {
                        x: x + sc(40),
                        y: y + sc(16),
                    },
                    MONITOR_DEFAULTTONULL,
                )
                .is_null()
            })
            .unwrap_or((CW_USEDEFAULT, CW_USEDEFAULT));

        // Kepemilikan satu ref Rc berpindah ke jendela hingga WM_NCDESTROY; ref lokal dilepas setelah inisialisasi.
        let ui = Rc::new(RefCell::new(Ui::new(&s)));
        let raw = Rc::into_raw(ui.clone());
        let title = wide(APP);
        let hwnd = CreateWindowExW(
            0,
            class.as_ptr(),
            title.as_ptr(),
            WINDOW_STYLE,
            x,
            y,
            rc.right - rc.left,
            rc.bottom - rc.top,
            null_mut(),
            null_mut(),
            inst,
            raw as *mut c_void,
        );
        if hwnd.is_null() {
            // Bila WM_NCDESTROY tidak pernah dipanggil, bebaskan raw ref yang belum diserahkan.
            if Rc::strong_count(&ui) == 2 {
                drop(Rc::from_raw(raw));
            }
            return false;
        }
        build(hwnd, &ui, &s);
        ShowWindow(hwnd, SW_SHOW);
        if start {
            toggle_run(hwnd, &ui);
        }
        drop(ui);

        let mut msg: MSG = std::mem::zeroed();
        let mut focus: HWND = null_mut();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            // IsDialogMessage memberi navigasi Tab/Shift+Tab, Spasi, dan Esc.
            if IsDialogMessageW(hwnd, &msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            let f = GetFocus();
            if f != focus {
                let old = focus;
                focus = f;
                if let Some(ui) = ui_of(hwnd) {
                    focus_changed(hwnd, &ui, old, f);
                }
            }
        }
        true
    }
}
