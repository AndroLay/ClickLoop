//! Pelukisan jendela: latar, kartu, teks, kontrol owner-draw, dan mode mini.
use std::ptr::null_mut;

use crate::clicker::{PAUSE_FOCUS_LOST, PAUSE_WAIT_TARGET};
use crate::config::MIN_POINTS;
use crate::r#macro;
use crate::theme::{self as th, Look, State};
use crate::win32::*;

use super::layout::*;
use super::window::Ui;
use super::APP;

/// Teks dan warna pil sesuai keadaan.
fn pill_state(ui: &Ui) -> (&'static str, u32, u32) {
    let p = th::pal();
    if ui.recording {
        ("Recording", p.idle_bg, p.warn)
    } else if ui.picking {
        ("Picking", p.info_bg, p.accent)
    } else if ui.player.is_playing() {
        if ui.player.countdown_ms() > 0 {
            ("Starting", p.info_bg, p.accent)
        } else {
            match ui.player.pause_reason() {
                PAUSE_WAIT_TARGET => ("Waiting", p.idle_bg, p.warn),
                PAUSE_FOCUS_LOST => ("Paused", p.idle_bg, p.warn),
                _ => ("Playing", p.ok_bg, p.ok_fg),
            }
        }
    } else if ui.runner.is_running() {
        if ui.runner.countdown_ms() > 0 {
            ("Starting", p.info_bg, p.accent)
        } else {
            match ui.runner.pause_reason() {
                PAUSE_WAIT_TARGET => ("Waiting", p.idle_bg, p.warn),
                PAUSE_FOCUS_LOST => ("Paused", p.idle_bg, p.warn),
                _ if ui.run_hold => ("Holding", p.ok_bg, p.ok_fg),
                _ => ("Running", p.ok_bg, p.ok_fg),
            }
        }
    } else {
        ("Ready", p.idle_bg, p.muted)
    }
}

/// Pil status, dipakai bersama mode penuh dan mini.
unsafe fn draw_pill(ui: &Ui, dc: HDC, clip: RECT) {
    let rc = rt(pill_of(ui.mini));
    if meets(&rc, &clip) {
        let (word, face, fg) = pill_state(ui);
        ui.theme.pill(dc, rc, word, face, fg, th::pal().bg);
    }
}

/// Teks di koordinat logis; hanya digambar bila menyentuh area yang sedang diperbarui.
unsafe fn text_at(
    dc: HDC,
    clip: RECT,
    at: (i32, i32, i32, i32),
    s: &str,
    font: HGDIOBJ,
    color: u32,
    flags: u32,
) {
    let rc = rt(at);
    if meets(&rc, &clip) {
        th::text(dc, s, rc, font, color, flags);
    }
}

fn trig_hint(trig: u32) -> String {
    match trig {
        0 => "Start hotkey starts and stops clicks.".to_string(),
        _ => format!("Clicks run while {} is held.", trig_name(trig)),
    }
}

fn pos_hint(ui: &Ui) -> String {
    match ui.pos {
        0 => "Follows cursor.".to_string(),
        1 => "Clicks at X, Y.".to_string(),
        _ if ui.points.len() < MIN_POINTS => format!("Pick {}+ points.", MIN_POINTS),
        _ => format!("{} points in order.", ui.points.len()),
    }
}

fn macro_hint(ui: &Ui) -> (String, u32) {
    let p = th::pal();
    if ui.recording {
        (format!("{} events", r#macro::recorded_count()), p.warn)
    } else if ui.macro_events.is_empty() {
        ("No macro".to_string(), p.subtle)
    } else {
        (r#macro::summary(&ui.macro_events), p.accent)
    }
}

/// Mode penuh: header (ikon, judul, tab switcher), kartu sesuai tab aktif, kontrol, dan footer.
unsafe fn draw_full(ui: &Ui, dc: HDC, clip: RECT) {
    let p = th::pal();
    let t = &ui.theme;
    let icon = r(14, 12, 26, 26);
    if !ui.icon.is_null() && meets(&icon, &clip) {
        DrawIconEx(
            dc,
            icon.left,
            icon.top,
            ui.icon,
            sc(26),
            sc(26),
            0,
            null_mut(),
            DI_NORMAL,
        );
    }
    text_at(dc, clip, (46, 12, 108, 26), APP, t.title, p.text, DT_LEFT);

    let tab_rc = rt(TAB_WELL_FULL);
    if meets(&tab_rc, &clip) {
        th::round_rect(dc, tab_rc, sc(th::CTRL_R), p.seg_bg, p.seg_edge, 1, p.bg);
    }

    let cards: &[(i32, i32, i32, i32, &str)] = if ui.tab == TAB_CLICKER {
        &CARDS_CLICKER
    } else {
        &CARDS_MACRO
    };
    for c in cards.iter() {
        let rc = r(c.0, c.1, c.2, c.3);
        if meets(&rc, &clip) {
            th::round_rect(dc, rc, sc(th::CARD_R), p.card, p.card_edge, 1, p.bg);
        }
        if !c.4.is_empty() {
            text_at(
                dc,
                clip,
                (c.0 + 12, c.1 + 8, c.2 - 24, 18),
                c.4,
                t.strong,
                p.text,
                DT_LEFT,
            );
        }
    }

    if ui.tab == TAB_CLICKER {
        let rate_col = if ui.rate_warn { p.warn } else { p.accent };
        text_at(
            dc,
            clip,
            RATE,
            &ui.rate,
            t.small,
            rate_col,
            DT_RIGHT | DT_END_ELLIPSIS,
        );
        text_at(
            dc,
            clip,
            DELAY_HINT_CLICKER,
            "0 = immediate",
            t.small,
            p.subtle,
            DT_RIGHT,
        );
        text_at(
            dc,
            clip,
            STOP_HINT,
            "0 = no limit",
            t.small,
            p.subtle,
            DT_RIGHT,
        );
        text_at(
            dc,
            clip,
            POS_HINT,
            &pos_hint(ui),
            t.small,
            p.subtle,
            DT_RIGHT | DT_END_ELLIPSIS,
        );
        text_at(
            dc,
            clip,
            TRIG_HINT,
            &trig_hint(ui.trig),
            t.small,
            p.subtle,
            DT_LEFT | DT_END_ELLIPSIS,
        );

        for l in LABELS_CLICKER.iter() {
            let color = if l.5 && ui.hold { p.disabled } else { p.muted };
            text_at(dc, clip, (l.1, l.2, l.3, l.4), l.0, t.small, color, DT_LEFT);
        }
        for &(x, y, w, hh) in &SEG_GROUPS[..4] {
            let rc = r(x, y, w, hh);
            if meets(&rc, &clip) {
                th::round_rect(dc, rc, sc(th::CTRL_R), p.seg_bg, p.seg_edge, 1, p.card);
            }
        }
    } else {
        let (mh, mcol) = macro_hint(ui);
        text_at(
            dc,
            clip,
            MACRO_HINT,
            &mh,
            t.small,
            mcol,
            DT_RIGHT | DT_END_ELLIPSIS,
        );
        text_at(
            dc,
            clip,
            DELAY_HINT_MACRO,
            "0 = immediate",
            t.small,
            p.subtle,
            DT_RIGHT,
        );
        text_at(
            dc,
            clip,
            STOP_HINT,
            "0 = no limit",
            t.small,
            p.subtle,
            DT_RIGHT,
        );
        for l in LABELS_MACRO.iter() {
            text_at(
                dc,
                clip,
                (l.1, l.2, l.3, l.4),
                l.0,
                t.small,
                p.muted,
                DT_LEFT | DT_END_ELLIPSIS,
            );
        }
        let (d1, d2, d3) = if ui.recording {
            (
                format!("Recording live: {} events", r#macro::recorded_count()),
                "Mouse movement, clicks, scroll & keys".to_string(),
                "Press Stop, hotkey, or Esc to finish".to_string(),
            )
        } else if ui.macro_events.is_empty() {
            (
                "No macro recorded or loaded yet.".to_string(),
                "Click Record or Load a .clm file.".to_string(),
                "Supports mouse, scroll & keyboard.".to_string(),
            )
        } else {
            let acts = r#macro::action_count(&ui.macro_events);
            let secs = r#macro::total_us(&ui.macro_events) as f64 / 1_000_000.0;
            let mode = if ui.macro_loop {
                "Loop until stopped"
            } else {
                "Run once"
            };
            (
                format!("{} events ({} actions)", ui.macro_events.len(), acts),
                format!("Duration: {secs:.2} s \u{B7} {mode}"),
                "Format: ClickLoop Macro (.clm)".to_string(),
            )
        };
        text_at(
            dc,
            clip,
            MACRO_DETAIL_1,
            &d1,
            t.small,
            p.text,
            DT_LEFT | DT_END_ELLIPSIS,
        );
        text_at(
            dc,
            clip,
            MACRO_DETAIL_2,
            &d2,
            t.small,
            p.muted,
            DT_LEFT | DT_END_ELLIPSIS,
        );
        text_at(
            dc,
            clip,
            MACRO_DETAIL_3,
            &d3,
            t.small,
            p.subtle,
            DT_LEFT | DT_END_ELLIPSIS,
        );
    }

    for (i, f) in FIELDS.iter().enumerate() {
        if !field_visible(ui.tab, i) {
            continue;
        }
        let rc = field_rect_for(ui.tab, i);
        if !meets(&rc, &clip) {
            continue;
        }
        let c = ui.ctl[i];
        let off = !c.is_null() && IsWindowEnabled(c) == 0;
        let focus = !c.is_null() && ui.focus == c;
        t.field(dc, rc, f.prefix, f.unit, focus, off);
    }

    let color = if ui.warn { p.warn } else { p.muted };
    text_at(
        dc,
        clip,
        STATUS,
        &ui.status,
        t.small,
        color,
        DT_LEFT | DT_END_ELLIPSIS,
    );
}

/// Mode mini: label mode aktif (Auto Clicker / Macro), status, well tab switcher, dan tombol sesuai mode.
unsafe fn draw_mini(ui: &Ui, dc: HDC, clip: RECT) {
    let p = th::pal();
    let mode_text = if ui.recording || ui.player.is_playing() {
        "Macro"
    } else if ui.runner.is_running() {
        "Clicker"
    } else if ui.tab == TAB_MACRO {
        "Macro"
    } else {
        "Clicker"
    };
    let mode_col = if ui.recording {
        p.warn
    } else if ui.player.is_playing() || ui.runner.is_running() {
        p.ok_fg
    } else {
        p.accent
    };
    text_at(
        dc,
        clip,
        MINI_MODE_LBL,
        mode_text,
        ui.theme.caption,
        mode_col,
        DT_LEFT | DT_END_ELLIPSIS,
    );

    let color = if ui.warn { p.warn } else { p.muted };
    text_at(
        dc,
        clip,
        status_of(true),
        &ui.status,
        ui.theme.small,
        color,
        DT_LEFT | DT_END_ELLIPSIS,
    );

    let tab_rc = rt(TAB_WELL_MINI);
    if meets(&tab_rc, &clip) {
        th::round_rect(dc, tab_rc, sc(th::CTRL_R), p.seg_bg, p.seg_edge, 1, p.bg);
    }
}

/// Hanya elemen yang menyentuh `clip` yang digambar, sehingga pembaruan kecil tidak menggambar ulang seluruh jendela.
unsafe fn draw_window(ui: &Ui, dc: HDC, clip: RECT) {
    th::fill(dc, clip, th::pal().bg);
    draw_pill(ui, dc, clip);
    if ui.mini {
        draw_mini(ui, dc, clip);
    } else {
        draw_full(ui, dc, clip);
    }
}

/// Menangani WM_PAINT.
pub(super) unsafe fn paint(h: HWND, ui: &Ui) {
    let mut ps: PAINTSTRUCT = std::mem::zeroed();
    let hdc = BeginPaint(h, &mut ps);
    if !hdc.is_null() {
        let buf = th::Buffer::new(hdc, ps.rc_paint);
        draw_window(ui, buf.dc(), ps.rc_paint);
    }
    EndPaint(h, &ps);
}

/// Indeks pilihan aktif untuk grup segmen `g` (urutan SEG_GROUPS).
fn selected(ui: &Ui, g: usize) -> usize {
    match g {
        0 => ui.button as usize,
        1 => ui.kind as usize,
        2 => ui.trig as usize,
        3 => ui.pos as usize,
        _ => ui.tab,
    }
}

/// Menggambar kontrol owner-draw: segmen, toggle, dan tombol.
pub(super) unsafe fn draw_item(ui: &Ui, d: &DrawItemStruct) {
    let p = th::pal();
    let st = State {
        down: d.item_state & ODS_SELECTED != 0,
        hot: ui.hot == d.hwnd_item,
        focus: d.item_state & ODS_FOCUS != 0 && d.item_state & ODS_NOFOCUSRECT == 0,
        off: d.item_state & ODS_DISABLED != 0,
    };
    let buf = th::Buffer::new(d.hdc, d.rc_item);
    let dc = buf.dc();
    let rc = d.rc_item;
    let caption = text_of(d.hwnd_item);
    let t = &ui.theme;
    let id = d.ctl_id as i32;
    let footer = matches!(id, ID_START | ID_MINI);
    let bg = if ui.mini || footer { p.bg } else { p.card };

    if let Some(g) = seg_group_of(id) {
        let k = (id - SEG_FIRST[g]) as usize;
        t.segment(dc, rc, &caption, selected(ui, g) == k, st);
        return;
    }
    match id {
        ID_HOLD => t.toggle(dc, rc, &caption, HOLD_HINT, ui.hold, st),
        ID_MACRO_LOOP => t.toggle(dc, rc, &caption, MACRO_LOOP_HINT, ui.macro_loop, st),
        ID_FOCUS => t.toggle(dc, rc, &caption, "", ui.focus_guard, st),
        ID_FAILSAFE => t.toggle(dc, rc, &caption, "", ui.failsafe, st),
        ID_TRAYOPT => t.toggle(dc, rc, &caption, "", ui.tray, st),
        ID_PICK => {
            let look = if ui.picking {
                Look::Primary
            } else {
                Look::Plain
            };
            t.button(dc, rc, look, th::GLYPH_PICK, &caption, st, bg, false)
        }
        ID_MACRO_REC => {
            let (look, glyph) = if ui.recording {
                (Look::Danger, th::GLYPH_STOP)
            } else {
                (Look::Plain, th::GLYPH_RECORD)
            };
            t.button(dc, rc, look, glyph, &caption, st, bg, false)
        }
        ID_MACRO_PLAY => {
            let (look, glyph) = if ui.player.is_playing() {
                (Look::Danger, th::GLYPH_STOP)
            } else {
                (Look::Primary, th::GLYPH_PLAY)
            };
            t.button(dc, rc, look, glyph, &caption, st, bg, false)
        }
        ID_START => {
            let (look, glyph) = if ui.tab == TAB_MACRO {
                if ui.recording || ui.player.is_playing() {
                    (Look::Danger, th::GLYPH_STOP)
                } else if ui.macro_events.is_empty() {
                    (Look::Primary, th::GLYPH_RECORD)
                } else {
                    (Look::Primary, th::GLYPH_PLAY)
                }
            } else if ui.runner.is_running() {
                (Look::Danger, th::GLYPH_STOP)
            } else {
                (Look::Primary, th::GLYPH_PLAY)
            };
            t.button(dc, rc, look, glyph, &caption, st, bg, false)
        }
        ID_PIN => {
            let glyph = if ui.topmost {
                th::GLYPH_PINNED
            } else {
                th::GLYPH_PIN
            };
            t.button(dc, rc, Look::Ghost, glyph, "", st, p.bg, ui.topmost)
        }
        ID_THEME => {
            let glyph = if th::is_dark() {
                th::GLYPH_SUN
            } else {
                th::GLYPH_MOON
            };
            t.button(dc, rc, Look::Ghost, glyph, "", st, p.bg, false)
        }
        ID_CLEAR | ID_LOAD | ID_SAVE | ID_MINI | ID_EXPAND | ID_MACRO_LOAD | ID_MACRO_SAVE
        | ID_MACRO_CLEAR => t.button(dc, rc, Look::Plain, "", &caption, st, bg, false),
        _ => {}
    }
}
