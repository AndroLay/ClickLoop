//! Pemetaan antara kontrol jendela dan `Settings`: membaca isian, menerapkan profil, dan teks kecepatan.
use crate::config::{Settings, LOW_INTERVAL_MS, MAX_DELAY_MS, MAX_MILLIS, MAX_POS_JITTER};
use crate::keys;
use crate::win32::*;

use super::actions::{
    redraw_all, refresh_enabled, refresh_macro_buttons, refresh_start, set_topmost,
};
use super::layout::*;
use super::window::{Ui, UiCell};

/// Teks awal kotak isian `i` (urutan FIELDS). Kotak hotkey tidak memakai fungsi ini.
pub(super) fn field_text(s: &Settings, i: usize) -> String {
    let (dm, ds, dms) = s.delay_parts();
    match i {
        0 => s.hours.to_string(),
        1 => s.minutes.to_string(),
        2 => s.seconds.to_string(),
        3 => s.millis.to_string(),
        4 => s.jitter.to_string(),
        5 => s.press_ms.to_string(),
        6 => s.max_clicks.to_string(),
        7 => s.max_secs.to_string(),
        8 => s.x.to_string(),
        9 => s.y.to_string(),
        12 => s.pos_jitter.to_string(),
        F_DELAY_MIN => dm.to_string(),
        F_DELAY_SEC => ds.to_string(),
        F_DELAY_MS => dms.to_string(),
        _ => String::new(),
    }
}

/// Indeks segmen posisi (0 Cursor, 1 Fixed, 2 Sequence) dari `Settings`.
pub(super) fn pos_of(s: &Settings) -> u32 {
    if s.sequence {
        2
    } else if s.fixed {
        1
    } else {
        0
    }
}

/// Indeks segmen pemicu (0 Hotkey, 1 Mouse 4, 2 Mouse 5) dari `Settings`.
pub(super) fn trig_of(s: &Settings) -> u32 {
    if s.trigger_hold {
        1 + s.trigger_btn.min(1)
    } else {
        0
    }
}

/// Membaca angka dari kotak; kosong memakai batas bawah, tidak valid memakai `def`.
unsafe fn num(c: HWND, lo: u32, hi: u32, def: u32) -> u32 {
    let t = text_of(c);
    let t = t.trim();
    if t.is_empty() {
        return lo;
    }
    t.parse::<u64>()
        .map(|n| n.clamp(lo as u64, hi as u64) as u32)
        .unwrap_or(def)
}

unsafe fn int(c: HWND, def: i32) -> i32 {
    text_of(c).trim().parse::<i32>().unwrap_or(def)
}

/// Pengaturan saat ini dari seluruh kontrol. Isian yang tidak valid memakai nilai dari `ui.base`.
pub(super) unsafe fn read_settings(ui: &Ui) -> Settings {
    let d = &ui.base;
    let c = &ui.ctl;
    let (def_dm, def_ds, def_dms) = d.delay_parts();
    let dm = num(c[F_DELAY_MIN], 0, 99, def_dm) as u64;
    let ds = num(c[F_DELAY_SEC], 0, 99, def_ds) as u64;
    let dms = num(c[F_DELAY_MS], 0, MAX_MILLIS, def_dms) as u64;
    let delay_ms = (dm * 60_000 + ds * 1000 + dms).min(MAX_DELAY_MS as u64) as u32;
    Settings {
        hours: num(c[0], 0, 99, d.hours),
        minutes: num(c[1], 0, 99, d.minutes),
        seconds: num(c[2], 0, 99, d.seconds),
        millis: num(c[3], 0, MAX_MILLIS, d.millis),
        jitter: num(c[4], 0, 9999, d.jitter),
        press_ms: num(c[5], 0, 60_000, d.press_ms),
        max_clicks: num(c[6], 0, u32::MAX, d.max_clicks),
        max_secs: num(c[7], 0, u32::MAX, d.max_secs),
        x: int(c[F_X], d.x),
        y: int(c[F_Y], d.y),
        pos_jitter: num(c[F_POS_JITTER], 0, MAX_POS_JITTER, d.pos_jitter),
        button: ui.button,
        kind: ui.kind,
        hold: ui.hold,
        fixed: ui.pos == 1,
        sequence: ui.pos == 2,
        points: ui.points.clone(),
        delay_ms,
        macro_loop: ui.macro_loop,
        focus_guard: ui.focus_guard,
        failsafe: ui.failsafe,
        trigger_hold: ui.trig != 0,
        trigger_btn: ui.trig.saturating_sub(1),
        tray: ui.tray,
        start_key: ui.start_key,
        pick_key: ui.pick_key,
        rec_key: ui.rec_key,
        play_key: ui.play_key,
        topmost: ui.topmost,
        dark: ui.dark,
        win_pos: ui.win_pos,
    }
}

/// Menerapkan profil ke seluruh kontrol. Posisi jendela, tema, mode mini, dan hotkey aktif tidak diubah di sini;
/// pemanggil wajib memanggil `apply_hotkeys` setelahnya.
pub(super) unsafe fn apply_settings(h: HWND, ui: &UiCell, s: &Settings) {
    let ctls = {
        let mut u = ui.borrow_mut();
        u.button = s.button;
        u.kind = s.kind;
        u.trig = trig_of(s);
        u.pos = pos_of(s);
        u.hold = s.hold;
        u.macro_loop = s.macro_loop;
        u.focus_guard = s.focus_guard;
        u.failsafe = s.failsafe;
        u.tray = s.tray;
        u.topmost = s.topmost;
        u.points = s.points.clone();
        u.start_key = s.start_key;
        u.pick_key = s.pick_key;
        u.rec_key = s.rec_key;
        u.play_key = s.play_key;
        u.base = s.clone();
        u.ctl
    };
    // SetWindowTextW pada EDIT memicu EN_CHANGE secara sinkron; borrow harus sudah dilepas.
    for i in [
        0,
        1,
        2,
        3,
        4,
        5,
        6,
        7,
        8,
        9,
        12,
        F_DELAY_MIN,
        F_DELAY_SEC,
        F_DELAY_MS,
    ] {
        set_text(ctls[i], &field_text(s, i));
    }
    SendMessageW(
        ctls[F_HK_START],
        HKM_SETHOTKEY,
        keys::to_word(s.start_key),
        0,
    );
    SendMessageW(ctls[F_HK_PICK], HKM_SETHOTKEY, keys::to_word(s.pick_key), 0);
    SendMessageW(ctls[F_HK_REC], HKM_SETHOTKEY, keys::to_word(s.rec_key), 0);
    SendMessageW(ctls[F_HK_PLAY], HKM_SETHOTKEY, keys::to_word(s.play_key), 0);
    set_topmost(h, s.topmost);

    refresh_enabled(h, ui);
    refresh_start(ui);
    refresh_macro_buttons(ui);
    update_rate(h, ui);
    redraw_all(h, ui);
}

/// Pesan peringatan bila interval total di bawah ambang rendah (`LOW_INTERVAL_MS`).
pub(super) fn low_interval_warning(ms: u64) -> Option<String> {
    if ms >= LOW_INTERVAL_MS {
        return None;
    }
    if ms == 0 {
        Some(
            "Warning: 0 ms runs at 1 ms min (very fast). Enable Failsafe to stay in control."
                .to_string(),
        )
    } else {
        Some(format!(
            "Warning: {ms} ms is very fast (< {LOW_INTERVAL_MS} ms) and may lag target apps."
        ))
    }
}

/// Teks kecepatan di kartu interval, misalnya "≈ 10 clicks/s" atau "1 click every 2.5 s".
/// Mengembalikan `Some(ms)` bila bukan mode hold dan interval di bawah `LOW_INTERVAL_MS`.
pub(super) unsafe fn update_rate(h: HWND, ui: &UiCell) -> Option<u64> {
    let (changed, low_ms) = {
        let mut u = ui.borrow_mut();
        let (text, warn, low_ms) = if u.hold {
            ("Hold mode".to_string(), false, None)
        } else {
            let s = read_settings(&u);
            let ms = s.interval_ms();
            let rate = rate_text(ms, s.clicks_per_action() as u64);
            if ms < LOW_INTERVAL_MS {
                (format!("Very fast \u{B7} {rate}"), true, Some(ms))
            } else {
                (rate, false, None)
            }
        };
        let changed = text != u.rate || warn != u.rate_warn;
        if changed {
            u.rate = text;
            u.rate_warn = warn;
        }
        (changed, low_ms)
    };
    if changed && !ui.borrow().mini && ui.borrow().tab == TAB_CLICKER {
        InvalidateRect(h, &rt(RATE), 0);
    }
    low_ms
}

fn rate_text(ms: u64, per: u64) -> String {
    let ms = ms.max(1);
    if ms < 1000 {
        let tenths = 10_000 * per / ms;
        if tenths >= 1000 {
            format!("\u{2248} {} clicks/s", tenths / 10)
        } else {
            format!("\u{2248} {}.{} clicks/s", tenths / 10, tenths % 10)
        }
    } else {
        let every = span(ms);
        if per == 1 {
            format!("1 click every {every}")
        } else {
            format!("{per} clicks every {every}")
        }
    }
}

/// Durasi ringkas: "2.5 s", "3 min 20 s", "1 h 5 min".
fn span(ms: u64) -> String {
    if ms < 60_000 {
        let (s, d) = (ms / 1000, ms % 1000 / 100);
        if d == 0 {
            format!("{s} s")
        } else {
            format!("{s}.{d} s")
        }
    } else if ms < 3_600_000 {
        let (m, s) = (ms / 60_000, ms % 60_000 / 1000);
        if s == 0 {
            format!("{m} min")
        } else {
            format!("{m} min {s} s")
        }
    } else {
        let (hh, m) = (ms / 3_600_000, ms % 3_600_000 / 60_000);
        if m == 0 {
            format!("{hh} h")
        } else {
            format!("{hh} h {m} min")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_text_and_span_formatting() {
        assert_eq!(rate_text(100, 1), "\u{2248} 10.0 clicks/s");
        assert_eq!(rate_text(10, 1), "\u{2248} 100 clicks/s");
        assert_eq!(rate_text(0, 1), "\u{2248} 1000 clicks/s");
        assert_eq!(rate_text(2500, 1), "1 click every 2.5 s");
        assert_eq!(rate_text(2000, 3), "3 clicks every 2 s");
        assert_eq!(span(120_000), "2 min");
        assert_eq!(span(135_000), "2 min 15 s");
        assert_eq!(span(3_600_000), "1 h");
        assert_eq!(span(3_900_000), "1 h 5 min");

        assert!(low_interval_warning(0).is_some());
        assert!(low_interval_warning(1).is_some());
        assert!(low_interval_warning(9).is_some());
        assert!(low_interval_warning(10).is_none());
        assert!(low_interval_warning(9999).is_none());
    }

    #[test]
    fn setting_index_helpers_map_expected_segments() {
        let mut s = Settings::default();
        assert_eq!(pos_of(&s), 0);
        s.fixed = true;
        assert_eq!(pos_of(&s), 1);
        s.sequence = true;
        assert_eq!(pos_of(&s), 2);

        assert_eq!(trig_of(&s), 0);
        s.trigger_hold = true;
        s.trigger_btn = 0;
        assert_eq!(trig_of(&s), 1);
        s.trigger_btn = 1;
        assert_eq!(trig_of(&s), 2);
    }
}
