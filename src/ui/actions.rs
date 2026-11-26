//! Aksi pengguna: auto clicker, makro Record & Playback, perpindahan tab & mode mini, profil, dan hotkey.
use std::path::Path;
use std::ptr::{null, null_mut};
use std::time::{Duration, Instant};

use crate::clicker::{PAUSE_FOCUS_LOST, PAUSE_WAIT_TARGET, STOP_CORNER, STOP_SHAKE};
use crate::config::{self, MAX_POINTS, MIN_POINTS};
use crate::keys;
use crate::picker;
use crate::r#macro::{self, PlayConfig};
use crate::shell;
use crate::win32::*;

use super::create::{edit_inner_rect, hotkey_inner_rect};
use super::layout::*;
use super::settings::{apply_settings, read_settings};
use super::tray::sync_tip;
use super::window::{window_alive, Ui, UiCell};
use super::WINDOW_STYLE;

/// Cara klik atau playback berakhir; menentukan pesan status yang ditampilkan.
#[derive(Clone, Copy)]
pub(super) enum End {
    /// Dihentikan pengguna lewat tombol atau hotkey.
    User,
    /// Batas jumlah klik/waktu atau akhir makro tercapai.
    Limit,
    /// Failsafe menghentikan operasi.
    Failsafe,
}

fn end_from_reason(reason: u8) -> End {
    match reason {
        STOP_CORNER | STOP_SHAKE => End::Failsafe,
        _ => End::Limit,
    }
}

/// Format sisa hitung mundur dari milidetik: "3.0 s" atau "450 ms".
fn format_countdown(ms: u32) -> String {
    if ms >= 1000 {
        format!("{:.1} s", ms as f64 / 1000.0)
    } else {
        format!("{ms} ms")
    }
}

/// Aksi tombol utama footer (`ID_START`), otomatis menyesuaikan tab aktif (`Auto Clicker` vs `Record & Play`).
pub(super) unsafe fn primary_action(h: HWND, ui: &UiCell) {
    let (tab, rec, playing, has_macro) = {
        let u = ui.borrow();
        (
            u.tab,
            u.recording,
            u.player.is_playing() || u.was_playing,
            !u.macro_events.is_empty(),
        )
    };
    if tab == TAB_CLICKER {
        toggle_run(h, ui);
    } else if rec {
        finish_record(h, ui);
    } else if playing {
        finish_play(h, ui, End::User);
    } else if has_macro {
        toggle_play(h, ui);
    } else {
        toggle_record(h, ui);
    }
}

/// Memulai atau menghentikan klik otomatis. Pengaturan disimpan setiap kali mulai.
pub(super) unsafe fn toggle_run(h: HWND, ui: &UiCell) {
    let (running, was_running, stop_reason, hold, pos, pts_len, picking, recording, playing, tab) = {
        let u = ui.borrow();
        (
            u.runner.is_running(),
            u.was_running,
            u.runner.stop_reason(),
            u.hold,
            u.pos,
            u.points.len(),
            u.picking,
            u.recording,
            u.player.is_playing() || u.was_playing,
            u.tab,
        )
    };
    if running {
        finish_run(h, ui, End::User);
        return;
    }
    if was_running {
        finish_run(h, ui, end_from_reason(stop_reason));
        return;
    }
    if recording {
        finish_record(h, ui);
    }
    if playing {
        finish_play(h, ui, End::User);
    }
    if !hold && pos == 2 && pts_len < MIN_POINTS {
        set_status(
            h,
            ui,
            &format!("Pick at least {MIN_POINTS} points before starting."),
            true,
        );
        return;
    }
    if picking {
        end_pick(h, ui);
    }
    if tab != TAB_CLICKER {
        ui.borrow_mut().tab = TAB_CLICKER;
        apply_view(h, ui);
    }
    let s = read_settings(&ui.borrow());
    let _ = config::save(&s);
    let started = {
        let mut u = ui.borrow_mut();
        u.run_hold = s.hold;
        u.run_button = s.button;
        if u.runner.start(s) {
            u.started = Some(Instant::now());
            u.was_running = true;
            u.last_pill = (u.runner.countdown_ms() > 0, u.runner.pause_reason());
            true
        } else {
            false
        }
    };
    if !started {
        set_status(h, ui, "Could not start the click thread.", true);
        return;
    }
    SetTimer(h, TIMER_UI, TIMER_MS, null());
    refresh_start(ui);
    invalidate_pill(h, ui);
    running_status(h, ui);
}

/// Baris status saat auto clicker berjalan.
pub(super) unsafe fn running_status(h: HWND, ui: &UiCell) {
    let msg = {
        let u = ui.borrow();
        let cd = u.runner.countdown_ms();
        let pause = u.runner.pause_reason();
        if cd > 0 {
            format!("Starting in {}...", format_countdown(cd))
        } else if pause == PAUSE_WAIT_TARGET {
            "Waiting for target window...".to_string()
        } else if pause == PAUSE_FOCUS_LOST {
            "Paused \u{B7} target window unfocused".to_string()
        } else {
            let t = clock(u.started.map(|s| s.elapsed()).unwrap_or_default());
            if u.run_hold {
                let name = BUTTON_NAMES[(u.run_button as usize).min(BUTTON_NAMES.len() - 1)];
                format!("Holding {} \u{B7} {t}", name.to_lowercase())
            } else {
                format!(
                    "Clicker \u{B7} {} clicks \u{B7} {t}",
                    group(u.runner.clicks())
                )
            }
        }
    };
    set_status(h, ui, &msg, false);
}

/// Menghentikan auto clicker dan mencatat hasilnya di baris status.
unsafe fn finish_run(h: HWND, ui: &UiCell, end: End) {
    let (msg, warn) = {
        let mut u = ui.borrow_mut();
        u.runner.stop();
        u.was_running = false;
        u.last_pill = (false, 0);
        let t = clock(u.started.take().map(|s| s.elapsed()).unwrap_or_default());
        let n = group(u.runner.clicks());
        match (end, u.run_hold) {
            (End::Failsafe, true) => (format!("Failsafe released button after {t}."), true),
            (End::Failsafe, false) => (format!("Failsafe stopped \u{B7} {n} clicks in {t}"), true),
            (_, true) => (format!("Released after {t}."), false),
            (End::Limit, false) => (format!("Finished \u{B7} {n} clicks in {t}"), false),
            (End::User, false) => (format!("Stopped \u{B7} {n} clicks in {t}"), false),
        }
    };
    KillTimer(h, TIMER_UI);
    set_status(h, ui, &msg, warn);
    refresh_start(ui);
    invalidate_pill(h, ui);
}

// ---- Record & Playback Makro ----

/// Menggambar ulang area info dan detail makro pada Tab 1.
pub(super) unsafe fn redraw_macro_info(h: HWND, ui: &UiCell) {
    let (mini, tab) = {
        let u = ui.borrow();
        (u.mini, u.tab)
    };
    if !mini && tab == TAB_MACRO {
        redraw_rect(h, MACRO_HINT);
        redraw_rect(h, MACRO_DETAIL_1);
        redraw_rect(h, MACRO_DETAIL_2);
        redraw_rect(h, MACRO_DETAIL_3);
    }
}

/// Mulai atau hentikan perekaman makro.
pub(super) unsafe fn toggle_record(h: HWND, ui: &UiCell) {
    if ui.borrow().recording {
        finish_record(h, ui);
        return;
    }
    let (running, playing, picking, tab) = {
        let u = ui.borrow();
        (
            u.runner.is_running() || u.was_running,
            u.player.is_playing() || u.was_playing,
            u.picking,
            u.tab,
        )
    };
    if running {
        finish_run(h, ui, End::User);
    }
    if playing {
        finish_play(h, ui, End::User);
    }
    if picking {
        end_pick(h, ui);
    }
    if tab != TAB_MACRO {
        ui.borrow_mut().tab = TAB_MACRO;
        apply_view(h, ui);
    }
    let skip_keys = {
        let u = ui.borrow();
        [u.start_key, u.rec_key, u.play_key]
    };
    if !r#macro::start_recording(skip_keys) {
        set_status(
            h,
            ui,
            "Could not start recording (input hook unavailable).",
            true,
        );
        return;
    }
    let rk = {
        let mut u = ui.borrow_mut();
        u.recording = true;
        u.started = Some(Instant::now());
        u.rec_key
    };
    RegisterHotKey(h, HK_ESC, MOD_NOREPEAT, VK_ESCAPE);
    SetTimer(h, TIMER_UI, TIMER_MS, null());
    refresh_start(ui);
    refresh_macro_buttons(ui);
    refresh_enabled(h, ui);
    redraw_macro_info(h, ui);
    invalidate_pill(h, ui);
    set_status(
        h,
        ui,
        &format!(
            "Recording mouse & keys... Press {} or Esc to stop.",
            keys::name(rk)
        ),
        false,
    );
}

/// Mengakhiri perekaman dan menyimpan hasilnya ke memori UI.
pub(super) unsafe fn finish_record(h: HWND, ui: &UiCell) {
    let events = r#macro::stop_recording();
    UnregisterHotKey(h, HK_ESC);
    KillTimer(h, TIMER_UI);
    let (msg, warn) = {
        let mut u = ui.borrow_mut();
        u.recording = false;
        u.started = None;
        if events.is_empty() {
            ("Recording stopped (no actions captured).".to_string(), true)
        } else {
            let sum = r#macro::summary(&events);
            u.macro_events = events;
            (format!("Macro recorded \u{B7} {sum}"), false)
        }
    };
    refresh_start(ui);
    refresh_macro_buttons(ui);
    refresh_enabled(h, ui);
    redraw_macro_info(h, ui);
    invalidate_pill(h, ui);
    set_status(h, ui, &msg, warn);
}

/// Memulai atau menghentikan pemutaran makro.
pub(super) unsafe fn toggle_play(h: HWND, ui: &UiCell) {
    let (playing, was_playing, stop_reason, running, recording, picking, tab) = {
        let u = ui.borrow();
        (
            u.player.is_playing(),
            u.was_playing,
            u.player.stop_reason(),
            u.runner.is_running() || u.was_running,
            u.recording,
            u.picking,
            u.tab,
        )
    };
    if playing {
        finish_play(h, ui, End::User);
        return;
    }
    if was_playing {
        finish_play(h, ui, end_from_reason(stop_reason));
        return;
    }
    if recording {
        finish_record(h, ui);
    }
    if running {
        finish_run(h, ui, End::User);
    }
    if picking {
        end_pick(h, ui);
    }
    if ui.borrow().macro_events.is_empty() {
        set_status(h, ui, "Record or load a macro before playing.", true);
        return;
    }
    if tab != TAB_MACRO {
        ui.borrow_mut().tab = TAB_MACRO;
        apply_view(h, ui);
    }
    let s = read_settings(&ui.borrow());
    let _ = config::save(&s);
    let started = {
        let mut u = ui.borrow_mut();
        let cfg = PlayConfig {
            events: u.macro_events.clone(),
            loop_play: u.macro_loop,
            delay_ms: s.delay_ms,
            max_secs: s.max_secs,
            focus_guard: s.focus_guard,
            failsafe: s.failsafe,
        };
        if u.player.start(cfg) {
            u.started = Some(Instant::now());
            u.was_playing = true;
            u.last_pill = (u.player.countdown_ms() > 0, u.player.pause_reason());
            true
        } else {
            false
        }
    };
    if !started {
        set_status(h, ui, "Could not start macro playback.", true);
        return;
    }
    RegisterHotKey(h, HK_ESC, MOD_NOREPEAT, VK_ESCAPE);
    SetTimer(h, TIMER_UI, TIMER_MS, null());
    refresh_start(ui);
    refresh_macro_buttons(ui);
    refresh_enabled(h, ui);
    invalidate_pill(h, ui);
    playing_status(h, ui);
}

/// Baris status saat pemutaran makro berjalan.
pub(super) unsafe fn playing_status(h: HWND, ui: &UiCell) {
    let msg = {
        let u = ui.borrow();
        let cd = u.player.countdown_ms();
        let pause = u.player.pause_reason();
        if cd > 0 {
            format!("Starting macro in {}...", format_countdown(cd))
        } else if pause == PAUSE_WAIT_TARGET {
            "Waiting for target window...".to_string()
        } else if pause == PAUSE_FOCUS_LOST {
            "Macro paused \u{B7} target window unfocused".to_string()
        } else {
            let t = clock(u.started.map(|s| s.elapsed()).unwrap_or_default());
            if u.macro_loop {
                format!("Macro \u{B7} loop {} \u{B7} {t}", u.player.loops() + 1)
            } else {
                format!("Macro \u{B7} playing \u{B7} {t}")
            }
        }
    };
    set_status(h, ui, &msg, false);
}

/// Menghentikan pemutaran makro dan memperbarui UI.
pub(super) unsafe fn finish_play(h: HWND, ui: &UiCell, end: End) {
    let (msg, warn) = {
        let mut u = ui.borrow_mut();
        u.player.stop();
        u.was_playing = false;
        u.last_pill = (false, 0);
        let t = clock(u.started.take().map(|s| s.elapsed()).unwrap_or_default());
        let loops = u.player.loops();
        match end {
            End::Failsafe => (format!("Failsafe stopped playback after {t}."), true),
            End::Limit => {
                if loops > 1 {
                    (format!("Macro finished \u{B7} {loops} loops in {t}"), false)
                } else {
                    (format!("Macro finished in {t}."), false)
                }
            }
            End::User => (format!("Playback stopped after {t}."), false),
        }
    };
    UnregisterHotKey(h, HK_ESC);
    KillTimer(h, TIMER_UI);
    refresh_start(ui);
    refresh_macro_buttons(ui);
    refresh_enabled(h, ui);
    invalidate_pill(h, ui);
    set_status(h, ui, &msg, warn);
}

/// Memuat makro `.clm` dari dialog berkas.
pub(super) unsafe fn load_macro(h: HWND, ui: &UiCell) {
    let Some(path) = shell::macro_dialog(h, false) else {
        return;
    };
    if !window_alive(h) {
        return;
    }
    match r#macro::load_file(&path) {
        Ok(events) => {
            if ui.borrow().player.is_playing() || ui.borrow().was_playing {
                finish_play(h, ui, End::User);
            }
            let sum = r#macro::summary(&events);
            ui.borrow_mut().macro_events = events;
            refresh_start(ui);
            refresh_enabled(h, ui);
            redraw_macro_info(h, ui);
            set_status(
                h,
                ui,
                &format!("Loaded {} \u{B7} {sum}", profile_name(&path)),
                false,
            );
        }
        Err(e) => set_status(h, ui, &e, true),
    }
}

/// Menyimpan makro saat ini ke berkas `.clm`.
pub(super) unsafe fn save_macro(h: HWND, ui: &UiCell) {
    if ui.borrow().macro_events.is_empty() {
        set_status(h, ui, "Nothing to save. Record a macro first.", true);
        return;
    }
    let Some(path) = shell::macro_dialog(h, true) else {
        return;
    };
    if !window_alive(h) {
        return;
    }
    let events = ui.borrow().macro_events.clone();
    match r#macro::save_file(&path, &events) {
        Ok(()) => set_status(
            h,
            ui,
            &format!("Saved macro {}.", profile_name(&path)),
            false,
        ),
        Err(e) => set_status(h, ui, &format!("Could not save macro: {e}"), true),
    }
}

/// Menghapus makro yang ada di memori.
pub(super) unsafe fn clear_macro(h: HWND, ui: &UiCell) {
    if ui.borrow().player.is_playing() || ui.borrow().was_playing {
        finish_play(h, ui, End::User);
    }
    ui.borrow_mut().macro_events.clear();
    refresh_start(ui);
    refresh_enabled(h, ui);
    redraw_macro_info(h, ui);
    set_status(h, ui, "Macro cleared.", false);
}

/// Timer UI (250 ms) aktif saat merekam, memutar makro, atau menjalankan auto clicker.
pub(super) unsafe fn tick(h: HWND, ui: &UiCell) {
    let rec_status = {
        let u = ui.borrow();
        if u.recording {
            let t = clock(u.started.map(|s| s.elapsed()).unwrap_or_default());
            let n = r#macro::recorded_count();
            Some(format!(
                "Macro \u{B7} recording {} events \u{B7} {t}",
                group(n as u64)
            ))
        } else {
            None
        }
    };
    if let Some(msg) = rec_status {
        redraw_macro_info(h, ui);
        set_status(h, ui, &msg, false);
        return;
    }

    let play_state = {
        let mut u = ui.borrow_mut();
        if u.player.is_playing() {
            let pill = (u.player.countdown_ms() > 0, u.player.pause_reason());
            let pill_changed = pill != u.last_pill;
            if u.last_pill.0 && !pill.0 {
                u.started = Some(Instant::now());
            }
            if pill_changed {
                u.last_pill = pill;
            }
            Some((true, pill_changed, 0u8))
        } else if u.was_playing {
            Some((false, false, u.player.stop_reason()))
        } else {
            None
        }
    };
    if let Some((active, pill_changed, reason)) = play_state {
        if active {
            if pill_changed {
                invalidate_pill(h, ui);
            }
            playing_status(h, ui);
        } else {
            finish_play(h, ui, end_from_reason(reason));
        }
        return;
    }

    let run_state = {
        let mut u = ui.borrow_mut();
        if u.runner.is_running() {
            let pill = (u.runner.countdown_ms() > 0, u.runner.pause_reason());
            let pill_changed = pill != u.last_pill;
            if u.last_pill.0 && !pill.0 {
                u.started = Some(Instant::now());
            }
            if pill_changed {
                u.last_pill = pill;
            }
            Some((true, pill_changed, 0u8))
        } else if u.was_running {
            Some((false, false, u.runner.stop_reason()))
        } else {
            None
        }
    };
    match run_state {
        Some((true, pill_changed, _)) => {
            if pill_changed {
                invalidate_pill(h, ui);
            }
            running_status(h, ui);
        }
        Some((false, _, reason)) => {
            finish_run(h, ui, end_from_reason(reason));
        }
        None => {}
    }
}

/// Posisi dari tombol Pick atau hotkey. Mode urutan menambah titik; mode lain menjadi posisi tetap.
pub(super) unsafe fn set_pos(h: HWND, ui: &UiCell, x: i32, y: i32) {
    let (fx, fy) = {
        let u = ui.borrow();
        (u.ctl[F_X], u.ctl[F_Y])
    };
    set_text(fx, &x.to_string());
    set_text(fy, &y.to_string());
    let (msg, redraw_pos_hint, switched_fixed) = {
        let mut u = ui.borrow_mut();
        let note = next_start_note(&u);
        if u.pos == 2 {
            if u.points.len() < MAX_POINTS {
                u.points.push((x, y));
                (
                    format!("Added point {}/{}.{note}", u.points.len(), MAX_POINTS),
                    true,
                    false,
                )
            } else {
                (
                    format!("All {MAX_POINTS} points are set. Clear points to start over."),
                    false,
                    false,
                )
            }
        } else {
            let switched = u.pos != 1;
            if switched {
                u.pos = 1;
            }
            (format!("Position set to {x}, {y}.{note}"), false, switched)
        }
    };
    if redraw_pos_hint && ui.borrow().tab == TAB_CLICKER && !ui.borrow().mini {
        redraw_rect(h, POS_HINT);
    }
    if switched_fixed {
        redraw_group(ui, 3);
        refresh_enabled(h, ui);
    }
    set_status(h, ui, &msg, false);
}

/// Mode ambil posisi: klik di mana saja lewat hook mouse. Esc, klik kanan, atau klik tombol Cancel membatalkan.
pub(super) unsafe fn start_pick(h: HWND, ui: &UiCell) {
    if ui.borrow().recording {
        finish_record(h, ui);
    }
    let pick_btn = ui.borrow().pick_btn;
    if !picker::start(h, pick_btn) {
        set_status(
            h,
            ui,
            "Could not start picking (mouse hook unavailable).",
            true,
        );
        return;
    }
    ui.borrow_mut().picking = true;
    RegisterHotKey(h, HK_ESC, MOD_NOREPEAT, VK_ESCAPE);
    set_text(pick_btn, "Cancel");
    invalidate_pill(h, ui);
    set_status(
        h,
        ui,
        "Click anywhere to set the position. Right-click or Esc cancels.",
        false,
    );
}

/// Mengakhiri mode ambil posisi tanpa mengubah status.
pub(super) unsafe fn end_pick(h: HWND, ui: &UiCell) {
    picker::stop();
    UnregisterHotKey(h, HK_ESC);
    let pick_btn = {
        let mut u = ui.borrow_mut();
        u.picking = false;
        u.pick_btn
    };
    set_text(pick_btn, "Pick");
    invalidate_pill(h, ui);
}

/// Membatalkan mode ambil posisi bila sedang aktif.
pub(super) unsafe fn cancel_pick(h: HWND, ui: &UiCell) {
    if ui.borrow().picking {
        end_pick(h, ui);
        set_status(h, ui, "Picking cancelled.", false);
    }
}

/// Menghapus semua titik urutan.
pub(super) unsafe fn clear_points(h: HWND, ui: &UiCell) {
    ui.borrow_mut().points.clear();
    if !ui.borrow().mini && ui.borrow().tab == TAB_CLICKER {
        redraw_rect(h, POS_HINT);
    }
    set_status(h, ui, "Points cleared.", false);
}

/// Menutup: hentikan semua, lepas hotkey, lalu simpan pengaturan dan posisi jendela.
pub(super) unsafe fn shutdown(h: HWND, ui: &UiCell) {
    if ui.borrow().picking {
        end_pick(h, ui);
    }
    if ui.borrow().recording {
        let _ = r#macro::stop_recording();
        ui.borrow_mut().recording = false;
    }
    {
        let mut u = ui.borrow_mut();
        u.runner.stop();
        u.player.stop();
    }
    KillTimer(h, TIMER_UI);
    UnregisterHotKey(h, HK_START);
    UnregisterHotKey(h, HK_PICK);
    UnregisterHotKey(h, HK_REC);
    UnregisterHotKey(h, HK_PLAY);
    UnregisterHotKey(h, HK_ESC);
    if IsIconic(h) == 0 {
        let mut rc: RECT = std::mem::zeroed();
        if GetWindowRect(h, &mut rc) != 0 {
            ui.borrow_mut().win_pos = Some((rc.left, rc.top));
        }
    }
    let was_tray = {
        let mut u = ui.borrow_mut();
        let v = u.in_tray;
        u.in_tray = false;
        v
    };
    if was_tray {
        shell::tray_remove(h);
    }
    let s = read_settings(&ui.borrow());
    let _ = config::save(&s);
}

/// Pindah tab aktif (`TAB_CLICKER` atau `TAB_MACRO`).
pub(super) unsafe fn set_tab(h: HWND, ui: &UiCell, tab: usize) {
    let tab = tab.min(1);
    if ui.borrow().tab == tab {
        return;
    }
    if ui.borrow().picking {
        end_pick(h, ui);
    }
    ui.borrow_mut().tab = tab;
    apply_view(h, ui);
}

/// Mode penuh atau mini: mengatur ukuran jendela dan kontrol yang tampil.
pub(super) unsafe fn set_mode(h: HWND, ui: &UiCell, mini: bool) {
    if ui.borrow().picking {
        end_pick(h, ui);
    }
    ui.borrow_mut().mini = mini;
    apply_view(h, ui);
}

/// Menempatkan dan menampilkan/menyembunyikan seluruh kontrol sesuai `(ui.mini, ui.tab)`.
pub(super) unsafe fn apply_view(h: HWND, ui: &UiCell) {
    let (
        mini,
        tab,
        small,
        ctl,
        segs,
        hold_btn,
        pick_btn,
        clear_btn,
        focus_btn,
        failsafe_btn,
        tray_btn,
        load_btn,
        save_btn,
        macro_rec_btn,
        macro_play_btn,
        macro_load_btn,
        macro_save_btn,
        macro_clear_btn,
        macro_loop_btn,
        start_btn,
        mini_btn,
        pin_btn,
        theme_btn,
        expand_btn,
    ) = {
        let u = ui.borrow();
        (
            u.mini,
            u.tab,
            u.theme.small,
            u.ctl,
            u.segs,
            u.hold_btn,
            u.pick_btn,
            u.clear_btn,
            u.focus_btn,
            u.failsafe_btn,
            u.tray_btn,
            u.load_btn,
            u.save_btn,
            u.macro_rec_btn,
            u.macro_play_btn,
            u.macro_load_btn,
            u.macro_save_btn,
            u.macro_clear_btn,
            u.macro_loop_btn,
            u.start_btn,
            u.mini_btn,
            u.pin_btn,
            u.theme_btn,
            u.expand_btn,
        )
    };

    // Tab switcher selalu tampil (di header saat Expand, di baris 3 saat Mini).
    let (twx, twy, tww, twh) = tab_well_of(mini);
    let sw = (tww - 6) / 2;
    place(segs[SEG_BASE[4]], (twx + 3, twy + 3, sw, twh - 6));
    place(segs[SEG_BASE[4] + 1], (twx + 3 + sw, twy + 3, sw, twh - 6));
    ShowWindow(segs[SEG_BASE[4]], SW_SHOW);
    ShowWindow(segs[SEG_BASE[4] + 1], SW_SHOW);

    if mini {
        for &c in &ctl {
            ShowWindow(c, SW_HIDE);
        }
        for &s in &segs[..12] {
            ShowWindow(s, SW_HIDE);
        }
        for c in [
            hold_btn,
            pick_btn,
            clear_btn,
            focus_btn,
            failsafe_btn,
            tray_btn,
            load_btn,
            save_btn,
            macro_load_btn,
            macro_save_btn,
            macro_clear_btn,
            macro_loop_btn,
            mini_btn,
            theme_btn,
        ] {
            ShowWindow(c, SW_HIDE);
        }
        place(expand_btn, MINI_EXPAND);
        place(pin_btn, MINI_PIN);
        ShowWindow(expand_btn, SW_SHOW);
        ShowWindow(pin_btn, SW_SHOW);

        if tab == TAB_CLICKER {
            place(start_btn, MINI_START);
            ShowWindow(start_btn, SW_SHOW);
            ShowWindow(macro_rec_btn, SW_HIDE);
            ShowWindow(macro_play_btn, SW_HIDE);
            SetFocus(start_btn);
        } else {
            place(macro_rec_btn, MINI_REC);
            place(macro_play_btn, MINI_PLAY);
            ShowWindow(start_btn, SW_HIDE);
            ShowWindow(macro_rec_btn, SW_SHOW);
            ShowWindow(macro_play_btn, SW_SHOW);
            SetFocus(macro_rec_btn);
        }
    } else {
        ShowWindow(expand_btn, SW_HIDE);
        place(pin_btn, PIN_RC);
        place(theme_btn, THEME_RC);
        place(mini_btn, MINI_BTN_RC);
        place(start_btn, START_RC);
        ShowWindow(pin_btn, SW_SHOW);
        ShowWindow(theme_btn, SW_SHOW);
        ShowWindow(mini_btn, SW_SHOW);
        ShowWindow(start_btn, SW_SHOW);

        let is_clicker = tab == TAB_CLICKER;
        let clicker_cmd = if is_clicker { SW_SHOW } else { SW_HIDE };
        let macro_cmd = if is_clicker { SW_HIDE } else { SW_SHOW };

        for (i, &c) in ctl.iter().enumerate() {
            if field_visible(tab, i) {
                let inner = if matches!(i, F_HK_START | F_HK_PICK | F_HK_REC | F_HK_PLAY) {
                    hotkey_inner_rect(tab, i)
                } else {
                    edit_inner_rect(small, tab, i)
                };
                MoveWindow(
                    c,
                    inner.left,
                    inner.top,
                    inner.right - inner.left,
                    inner.bottom - inner.top,
                    1,
                );
                ShowWindow(c, SW_SHOW);
            } else {
                ShowWindow(c, SW_HIDE);
            }
        }
        for &s in &segs[..12] {
            ShowWindow(s, clicker_cmd);
        }
        for c in [hold_btn, pick_btn, clear_btn] {
            ShowWindow(c, clicker_cmd);
        }

        place(macro_rec_btn, MACRO_REC_RC);
        place(macro_play_btn, MACRO_PLAY_RC);
        for c in [
            macro_rec_btn,
            macro_play_btn,
            macro_load_btn,
            macro_save_btn,
            macro_clear_btn,
            macro_loop_btn,
        ] {
            ShowWindow(c, macro_cmd);
        }

        if is_clicker {
            place(focus_btn, FOCUS_RC_0);
            place(failsafe_btn, FAILSAFE_RC_0);
            place(tray_btn, TRAY_RC_0);
            place(load_btn, LOAD_RC_0);
            place(save_btn, SAVE_RC_0);
        } else {
            place(focus_btn, FOCUS_RC_1);
            place(failsafe_btn, FAILSAFE_RC_1);
            place(tray_btn, TRAY_RC_1);
            place(load_btn, LOAD_RC_1);
            place(save_btn, SAVE_RC_1);
        }
        for c in [focus_btn, failsafe_btn, tray_btn, load_btn, save_btn] {
            ShowWindow(c, SW_SHOW);
        }
    }

    let (cw, ch) = if mini { (MINI_W, MINI_H) } else { (W, H) };
    let mut rc = RECT {
        left: 0,
        top: 0,
        right: sc(cw),
        bottom: sc(ch),
    };
    adjust_rect(&mut rc, WINDOW_STYLE, dpi());
    SetWindowPos(
        h,
        null_mut(),
        0,
        0,
        rc.right - rc.left,
        rc.bottom - rc.top,
        SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
    );
    refresh_enabled(h, ui);
    refresh_start(ui);
    refresh_macro_buttons(ui);
    redraw_all(h, ui);
}

/// Semua kontrol anak jendela utama.
fn children(ui: &Ui) -> Vec<HWND> {
    let mut v = ui.ctl.to_vec();
    v.extend_from_slice(&ui.segs);
    v.extend([
        ui.hold_btn,
        ui.focus_btn,
        ui.failsafe_btn,
        ui.tray_btn,
        ui.pick_btn,
        ui.clear_btn,
        ui.load_btn,
        ui.save_btn,
        ui.mini_btn,
        ui.theme_btn,
        ui.macro_rec_btn,
        ui.macro_play_btn,
        ui.macro_load_btn,
        ui.macro_save_btn,
        ui.macro_clear_btn,
        ui.macro_loop_btn,
        ui.start_btn,
        ui.pin_btn,
        ui.expand_btn,
    ]);
    v
}

/// Memindahkan kontrol ke kotak logis `t`, dikonversi ke piksel sesuai DPI.
unsafe fn place(c: HWND, t: (i32, i32, i32, i32)) {
    let (x, y, w, hh) = t;
    MoveWindow(c, sc(x), sc(y), sc(w), sc(hh), 1);
}

/// Apakah isian `i` aktif: waktu tidak dipakai saat mode hold, dan koordinat hanya untuk posisi tetap.
fn field_on(ui: &Ui, i: usize) -> bool {
    if ui.hold && HOLD_OFF.contains(&i) {
        return false;
    }
    match i {
        F_X | F_Y => ui.pos == 1,
        F_POS_JITTER => {
            if ui.hold {
                ui.pos == 1
            } else {
                ui.pos != 0
            }
        }
        _ => true,
    }
}

/// Mengubah status aktif kontrol; mengembalikan `true` bila berubah.
unsafe fn set_enabled(c: HWND, on: bool, fallback: HWND) -> bool {
    if c.is_null() || (IsWindowEnabled(c) != 0) == on {
        return false;
    }
    if !on && GetFocus() == c {
        SetFocus(fallback);
    }
    EnableWindow(c, on as i32);
    true
}

/// Mengaktifkan atau menonaktifkan isian, segmen, dan tombol makro sesuai keadaan.
pub(super) unsafe fn refresh_enabled(h: HWND, ui: &UiCell) {
    let (
        tab,
        mini,
        ctl,
        segs,
        start_btn,
        hold,
        has_macro,
        recording,
        playing,
        m_play,
        m_save,
        m_clear,
        m_load,
        m_rec,
    ) = {
        let u = ui.borrow();
        let mut states = [true; 18];
        for (i, st) in states.iter_mut().enumerate() {
            *st = field_on(&u, i);
        }
        (
            u.tab,
            u.mini,
            (u.ctl, states),
            u.segs,
            u.start_btn,
            u.hold,
            !u.macro_events.is_empty(),
            u.recording,
            u.player.is_playing(),
            u.macro_play_btn,
            u.macro_save_btn,
            u.macro_clear_btn,
            u.macro_load_btn,
            u.macro_rec_btn,
        )
    };
    for (i, &c) in ctl.0.iter().enumerate() {
        if set_enabled(c, ctl.1[i], start_btn) && !mini && field_visible(tab, i) {
            InvalidateRect(h, &field_rect_for(tab, i), 0);
        }
    }
    let kind = SEG_BASE[1]..SEG_BASE[1] + seg_names(1).len();
    for &s in &segs[kind] {
        if set_enabled(s, !hold, start_btn) {
            redraw(s);
        }
    }
    let seq_seg = segs[SEG_BASE[3] + 2];
    if set_enabled(seq_seg, !hold, start_btn) {
        redraw(seq_seg);
    }
    for (btn, on) in [
        (m_rec, !playing),
        (m_play, (has_macro || playing) && !recording),
        (m_load, !recording && !playing),
        (m_save, has_macro && !recording),
        (m_clear, has_macro && !recording),
    ] {
        if set_enabled(btn, on, start_btn) {
            redraw(btn);
        }
    }
}

/// Menggambar ulang satu kontrol.
pub(super) unsafe fn redraw(c: HWND) {
    InvalidateRect(c, null(), 0);
}

/// Menggambar ulang area kotak logis `t` di jendela `h`.
pub(super) unsafe fn redraw_rect(h: HWND, t: (i32, i32, i32, i32)) {
    InvalidateRect(h, &rt(t), 0);
}

/// Menggambar ulang segmen grup `g`.
pub(super) unsafe fn redraw_group(ui: &UiCell, g: usize) {
    let segs = ui.borrow().segs;
    let base = SEG_BASE[g];
    for &s in &segs[base..base + seg_names(g).len()] {
        redraw(s);
    }
}

/// Menggambar ulang seluruh jendela beserta kontrolnya.
pub(super) unsafe fn redraw_all(h: HWND, ui: &UiCell) {
    let list = children(&ui.borrow());
    InvalidateRect(h, null(), 0);
    for c in list {
        redraw(c);
    }
}

/// Menggambar ulang pil status (dan label mode pada mode mini).
pub(super) unsafe fn invalidate_pill(h: HWND, ui: &UiCell) {
    let mini = ui.borrow().mini;
    InvalidateRect(h, &rt(pill_of(mini)), 0);
    if mini {
        InvalidateRect(h, &rt(MINI_MODE_LBL), 0);
    }
}

/// Menampilkan status di baris bawah (atau baris mini). Teks yang sama dikirim ke tooltip tray.
pub(super) unsafe fn set_status(h: HWND, ui: &UiCell, s: &str, warn: bool) {
    let mini = {
        let mut u = ui.borrow_mut();
        if u.status == s && u.warn == warn {
            return;
        }
        u.status = s.to_string();
        u.warn = warn;
        u.mini
    };
    InvalidateRect(h, &rt(status_of(mini)), 0);
    sync_tip(h, ui);
}

/// Teks tombol utama (`ID_START`) mengikuti tab aktif, keadaan, dan hotkey aktif.
pub(super) unsafe fn refresh_start(ui: &UiCell) {
    let (btn, text) = {
        let u = ui.borrow();
        let text = if u.tab == TAB_MACRO {
            if u.recording || u.player.is_playing() {
                "Stop".to_string()
            } else if u.macro_events.is_empty() {
                format!("Record ({})", keys::name(u.rec_key))
            } else {
                format!("Play ({})", keys::name(u.play_key))
            }
        } else if u.runner.is_running() {
            "Stop".to_string()
        } else {
            format!("Start ({})", keys::name(u.start_key))
        };
        (u.start_btn, text)
    };
    set_text(btn, &text);
    redraw(btn);
}

/// Teks tombol Record/Play mengikuti keadaan dan hotkey aktif (`Record (F9)` / `Play (F10)` / `Stop`).
pub(super) unsafe fn refresh_macro_buttons(ui: &UiCell) {
    let (rec_btn, rec_text, play_btn, play_text) = {
        let u = ui.borrow();
        let rt = if u.recording {
            "Stop".to_string()
        } else {
            format!("Record ({})", keys::name(u.rec_key))
        };
        let pt = if u.player.is_playing() {
            "Stop".to_string()
        } else {
            format!("Play ({})", keys::name(u.play_key))
        };
        (u.macro_rec_btn, rt, u.macro_play_btn, pt)
    };
    set_text(rec_btn, &rec_text);
    redraw(rec_btn);
    set_text(play_btn, &play_text);
    redraw(play_btn);
}

/// Menjadikan jendela selalu di atas atau tidak.
pub(super) unsafe fn set_topmost(h: HWND, on: bool) {
    let after = (if on { HWND_TOPMOST } else { HWND_NOTOPMOST }) as HWND;
    SetWindowPos(
        h,
        after,
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
    );
}

/// Mewarnai bilah judul sesuai mode lewat DWM.
pub(super) unsafe fn apply_titlebar(h: HWND, dark: bool) {
    let on: i32 = dark as i32;
    DwmSetWindowAttribute(
        h,
        DWMWA_USE_IMMERSIVE_DARK_MODE,
        &on as *const i32 as *const std::ffi::c_void,
        4,
    );
}

/// Mengganti mode terang/gelap dari tombol header.
pub(super) unsafe fn toggle_theme(h: HWND, ui: &UiCell) {
    let dark = {
        let mut u = ui.borrow_mut();
        u.dark = !u.dark;
        crate::theme::set_dark(u.dark);
        u.theme.refresh();
        u.dark
    };
    apply_titlebar(h, dark);
    redraw_all(h, ui);
    let msg = if dark {
        "Dark mode on."
    } else {
        "Light mode on."
    };
    set_status(h, ui, msg, false);
}

/// Mendaftarkan ulang semua hotkey global. Mengembalikan pesan galat bila kombinasi dipakai aplikasi lain.
pub(super) unsafe fn apply_hotkeys(h: HWND, ui: &UiCell) -> Option<String> {
    let (sk, pk, rk, plk) = {
        let u = ui.borrow();
        (u.start_key, u.pick_key, u.rec_key, u.play_key)
    };
    UnregisterHotKey(h, HK_START);
    UnregisterHotKey(h, HK_PICK);
    UnregisterHotKey(h, HK_REC);
    UnregisterHotKey(h, HK_PLAY);
    let start = RegisterHotKey(h, HK_START, keys::register_mods(sk), sk.vk) != 0;
    let pick = RegisterHotKey(h, HK_PICK, keys::register_mods(pk), pk.vk) != 0;
    let rec = RegisterHotKey(h, HK_REC, keys::register_mods(rk), rk.vk) != 0;
    let play = RegisterHotKey(h, HK_PLAY, keys::register_mods(plk), plk.vk) != 0;
    if !start {
        Some(format!(
            "{} is used by another app. Choose another start hotkey.",
            keys::name(sk)
        ))
    } else if !pick {
        Some(format!(
            "{} is used by another app. Choose another pick hotkey.",
            keys::name(pk)
        ))
    } else if !rec {
        Some(format!(
            "{} is used by another app. Choose another record hotkey.",
            keys::name(rk)
        ))
    } else if !play {
        Some(format!(
            "{} is used by another app. Choose another play hotkey.",
            keys::name(plk)
        ))
    } else {
        None
    }
}

/// Teks status saat siap, menyebut hotkey utama.
pub(super) fn ready_text(ui: &Ui) -> String {
    format!(
        "Ready. {} click \u{B7} {} pick \u{B7} {} rec \u{B7} {} play",
        keys::name(ui.start_key),
        keys::name(ui.pick_key),
        keys::name(ui.rec_key),
        keys::name(ui.play_key),
    )
}

/// Catatan tambahan bila pengaturan diubah saat sedang berjalan.
pub(super) fn next_start_note(ui: &Ui) -> &'static str {
    if ui.runner.is_running() || ui.player.is_playing() {
        " Applies next start."
    } else {
        ""
    }
}

/// Waktu berjalan: "M:SS", atau "H:MM:SS" setelah satu jam.
fn clock(d: Duration) -> String {
    let s = d.as_secs();
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

/// Angka dengan pemisah ribuan, misalnya "12,345".
fn group(n: u64) -> String {
    let digits = n.to_string();
    let len = digits.len();
    let mut out = String::with_capacity(len + len / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// Nama berkas profil/makro tanpa folder, untuk pesan status.
fn profile_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Memuat profil .ini dari dialog. Pengaturan langsung dipakai dan hotkey didaftarkan ulang.
pub(super) unsafe fn load_profile(h: HWND, ui: &UiCell) {
    let Some(path) = shell::profile_dialog(h, false) else {
        return;
    };
    if !window_alive(h) {
        return;
    }
    let Some(s) = config::read_file(&path) else {
        set_status(h, ui, "Could not read that profile.", true);
        return;
    };
    apply_settings(h, ui, &s);
    let note = next_start_note(&ui.borrow());
    match apply_hotkeys(h, ui) {
        Some(e) => set_status(h, ui, &e, true),
        None => set_status(
            h,
            ui,
            &format!("Loaded {}.{note}", profile_name(&path)),
            false,
        ),
    }
}

/// Menyimpan pengaturan saat ini ke profil .ini.
pub(super) unsafe fn save_profile(h: HWND, ui: &UiCell) {
    let Some(path) = shell::profile_dialog(h, true) else {
        return;
    };
    if !window_alive(h) {
        return;
    }
    let s = read_settings(&ui.borrow());
    match config::save_file(&path, &s) {
        Ok(()) => set_status(
            h,
            ui,
            &format!("Saved profile {}.", profile_name(&path)),
            false,
        ),
        Err(e) => set_status(h, ui, &format!("Could not save the profile: {e}"), true),
    }
}
