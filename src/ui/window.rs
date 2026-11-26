//! Keadaan jendela utama: handle kontrol, pengaturan aktif, makro, dan status klik.
use std::cell::RefCell;
use std::ptr::null_mut;
use std::rc::Rc;
use std::time::Instant;

use crate::clicker::Runner;
use crate::config::{Hotkey, Settings};
use crate::r#macro::{self, Player};
use crate::theme::Theme;
use crate::win32::*;

use super::load_icon;
use super::settings::{pos_of, trig_of};

/// Pemegang state UI yang aman terhadap pesan Win32 re-entrant dan dialog modal.
pub(super) type UiCell = Rc<RefCell<Ui>>;

/// State aplikasi, disimpan sebagai raw `Rc<RefCell<Ui>>` di GWLP_USERDATA jendela.
pub(super) struct Ui {
    pub(super) runner: Runner,
    pub(super) player: Player,
    pub(super) theme: Theme,
    /// Ikon 32 px untuk header.
    pub(super) icon: HICON,
    /// Ikon kecil untuk area notifikasi.
    pub(super) tray_icon: HICON,
    /// Kotak isian, sesuai urutan FIELDS.
    pub(super) ctl: [HWND; 18],
    /// Tombol segmen semua grup; grup g mulai di SEG_BASE[g].
    pub(super) segs: [HWND; 14],
    pub(super) hold_btn: HWND,
    pub(super) focus_btn: HWND,
    pub(super) failsafe_btn: HWND,
    pub(super) tray_btn: HWND,
    pub(super) pick_btn: HWND,
    pub(super) clear_btn: HWND,
    pub(super) load_btn: HWND,
    pub(super) save_btn: HWND,
    pub(super) start_btn: HWND,
    pub(super) pin_btn: HWND,
    pub(super) mini_btn: HWND,
    pub(super) expand_btn: HWND,
    pub(super) theme_btn: HWND,
    pub(super) macro_rec_btn: HWND,
    pub(super) macro_play_btn: HWND,
    pub(super) macro_load_btn: HWND,
    pub(super) macro_save_btn: HWND,
    pub(super) macro_clear_btn: HWND,
    pub(super) macro_loop_btn: HWND,
    /// Mode gelap aktif. Disimpan di clickloop.ini, bukan di profil.
    pub(super) dark: bool,
    /// Pilihan segmen: tombol mouse, tipe klik, pemicu, dan posisi.
    pub(super) button: u32,
    pub(super) kind: u32,
    pub(super) trig: u32,
    pub(super) pos: u32,
    pub(super) hold: bool,
    pub(super) macro_loop: bool,
    pub(super) focus_guard: bool,
    pub(super) failsafe: bool,
    pub(super) tray: bool,
    pub(super) topmost: bool,
    /// Titik mode Sequence (MIN_POINTS..=MAX_POINTS).
    pub(super) points: Vec<(i32, i32)>,
    /// Rangkaian event makro yang terekam atau dimuat dari `.clm`.
    pub(super) macro_events: Vec<r#macro::Event>,
    pub(super) start_key: Hotkey,
    pub(super) pick_key: Hotkey,
    pub(super) rec_key: Hotkey,
    pub(super) play_key: Hotkey,
    pub(super) win_pos: Option<(i32, i32)>,
    /// Pengaturan utuh dari awal; field tanpa kontrol di UI tetap dibawa saat disimpan.
    pub(super) base: Settings,
    /// Mode tab aktif: 0 = Auto Clicker, 1 = Record & Play.
    pub(super) tab: usize,
    /// Mode mini hanya berlaku untuk sesi ini, tidak disimpan.
    pub(super) mini: bool,
    /// Jendela sedang disembunyikan dan ikon ada di area notifikasi.
    pub(super) in_tray: bool,
    /// Kontrol owner-draw di bawah kursor (efek hover).
    pub(super) hot: HWND,
    pub(super) focus: HWND,
    pub(super) picking: bool,
    pub(super) recording: bool,
    /// Hotkey global dilepas sementara selama kotak hotkey sedang diisi.
    pub(super) hk_paused: bool,
    pub(super) run_hold: bool,
    pub(super) run_button: u32,
    pub(super) started: Option<Instant>,
    pub(super) was_running: bool,
    pub(super) was_playing: bool,
    /// Status pil terakhir saat berjalan/memutar: `(countdown_active, pause_reason)`.
    pub(super) last_pill: (bool, u8),
    pub(super) status: String,
    pub(super) warn: bool,
    pub(super) rate: String,
    pub(super) rate_warn: bool,
}

impl Ui {
    pub(super) unsafe fn new(s: &Settings) -> Self {
        // Palet harus sudah benar sebelum Theme::new membuat kuas.
        crate::theme::set_dark(s.dark);
        Ui {
            runner: Runner::new(),
            player: Player::new(),
            theme: Theme::new(),
            icon: load_icon(sc(28)),
            tray_icon: load_icon(GetSystemMetrics(SM_CXSMICON)),
            ctl: [null_mut(); 18],
            segs: [null_mut(); 14],
            hold_btn: null_mut(),
            focus_btn: null_mut(),
            failsafe_btn: null_mut(),
            tray_btn: null_mut(),
            pick_btn: null_mut(),
            clear_btn: null_mut(),
            load_btn: null_mut(),
            save_btn: null_mut(),
            start_btn: null_mut(),
            pin_btn: null_mut(),
            mini_btn: null_mut(),
            expand_btn: null_mut(),
            theme_btn: null_mut(),
            macro_rec_btn: null_mut(),
            macro_play_btn: null_mut(),
            macro_load_btn: null_mut(),
            macro_save_btn: null_mut(),
            macro_clear_btn: null_mut(),
            macro_loop_btn: null_mut(),
            dark: s.dark,
            button: s.button,
            kind: s.kind,
            trig: trig_of(s),
            pos: pos_of(s),
            hold: s.hold,
            macro_loop: s.macro_loop,
            focus_guard: s.focus_guard,
            failsafe: s.failsafe,
            tray: s.tray,
            topmost: s.topmost,
            points: s.points.clone(),
            macro_events: Vec::new(),
            start_key: s.start_key,
            pick_key: s.pick_key,
            rec_key: s.rec_key,
            play_key: s.play_key,
            win_pos: s.win_pos,
            base: s.clone(),
            tab: 0,
            mini: false,
            in_tray: false,
            hot: null_mut(),
            focus: null_mut(),
            picking: false,
            recording: false,
            hk_paused: false,
            run_hold: false,
            run_button: 0,
            started: None,
            was_running: false,
            was_playing: false,
            last_pill: (false, 0),
            status: String::new(),
            warn: false,
            rate: String::new(),
            rate_warn: false,
        }
    }
}

impl Drop for Ui {
    fn drop(&mut self) {
        unsafe {
            if !self.icon.is_null() {
                DestroyIcon(self.icon);
            }
            if !self.tray_icon.is_null() {
                DestroyIcon(self.tray_icon);
            }
        }
    }
}

/// Mengambil klon `Rc<RefCell<Ui>>` dari jendela. Hanya dipanggil dari utas UI.
/// Menjaga `Ui` tetap hidup walau jendela ditutup saat dialog modal sedang terbuka.
pub(super) unsafe fn ui_of(h: HWND) -> Option<UiCell> {
    let p = GetWindowLongPtrW(h, GWLP_USERDATA) as *const RefCell<Ui>;
    if p.is_null() {
        return None;
    }
    Rc::increment_strong_count(p);
    Some(Rc::from_raw(p))
}

/// Jendela masih melekat pada `Ui` (belum mencapai `WM_NCDESTROY`).
pub(super) unsafe fn window_alive(h: HWND) -> bool {
    GetWindowLongPtrW(h, GWLP_USERDATA) != 0
}
