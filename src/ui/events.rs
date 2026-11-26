//! Pesan jendela utama, perintah dari kontrol anak, dan validasi kotak hotkey.
use std::cell::RefCell;
use std::ptr::{null, null_mut};
use std::rc::Rc;

use crate::clicker::cursor_pos;
use crate::keys;
use crate::picker;
use crate::shell;
use crate::win32::*;

use super::actions::{
    apply_hotkeys, apply_view, cancel_pick, clear_macro, clear_points, end_pick, finish_play,
    finish_record, invalidate_pill, load_macro, load_profile, next_start_note, playing_status,
    primary_action, ready_text, redraw, redraw_group, redraw_macro_info, redraw_rect,
    refresh_enabled, refresh_macro_buttons, refresh_start, running_status, save_macro,
    save_profile, set_mode, set_pos, set_status, set_tab, set_topmost, shutdown, start_pick, tick,
    toggle_play, toggle_record, toggle_run, toggle_theme, End,
};
use super::draw::{draw_item, paint};
use super::layout::*;
use super::settings::{low_interval_warning, update_rate};
use super::tray::{on_tray_message, readd_tray, restore, to_tray};
use super::window::{ui_of, Ui, UiCell};
use super::{taskbar_created_msg, WM_RESTORE_APP};

/// Menghancurkan semua kontrol anak jendela.
unsafe fn destroy_children(h: HWND) {
    let mut c = GetWindow(h, GW_CHILD);
    while !c.is_null() {
        let next = GetWindow(c, GW_HWNDNEXT);
        DestroyWindow(c);
        c = next;
    }
}

/// Menerapkan DPI baru ke tema dan ikon jendela. Dipanggil setelah kontrol anak dihancurkan (atau sebelum dibuat)
/// agar HFONT/HBRUSH lama tidak pernah dilepas saat masih dipakai kontrol.
unsafe fn apply_dpi(h: HWND, ui: &UiCell, d: u32, rc: RECT) {
    set_dpi(d);
    {
        let mut u = ui.borrow_mut();
        // Font dan brush lama dilepas oleh Drop untuk Theme saat nilainya diganti.
        u.theme = crate::theme::Theme::new();
        DestroyIcon(u.icon);
        u.icon = super::load_icon(sc(32));
        u.hot = null_mut();
        u.focus = null_mut();
    }
    SetWindowPos(
        h,
        null_mut(),
        rc.left,
        rc.top,
        rc.right - rc.left,
        rc.bottom - rc.top,
        SWP_NOZORDER | SWP_NOACTIVATE,
    );
}

/// Membangun ulang seluruh kontrol dengan DPI baru secara aman: baca pengaturan, hancurkan kontrol lama dulu,
/// baru ganti Theme/DPI, bangun kontrol baru, dan pulihkan state (picking, recording, playing, running, mode mini).
unsafe fn rebuild_for_dpi(h: HWND, ui: &UiCell, d: u32, rc: RECT) {
    let (s, mini, picking, recording, was_playing, was_running) = {
        let u = ui.borrow();
        (
            super::settings::read_settings(&u),
            u.mini,
            u.picking,
            u.recording,
            u.was_playing,
            u.was_running,
        )
    };
    if picking {
        picker::stop();
        UnregisterHotKey(h, HK_ESC);
    }
    destroy_children(h);
    {
        ui.borrow_mut().hk_paused = false;
    }
    apply_dpi(h, ui, d, rc);
    super::create::build(h, ui, &s);
    apply_view(h, ui);
    if picking && !mini {
        start_pick(h, ui);
    } else if picking {
        ui.borrow_mut().picking = false;
    }
    if recording {
        refresh_start(ui);
        refresh_macro_buttons(ui);
        refresh_enabled(h, ui);
        invalidate_pill(h, ui);
    } else if was_playing && ui.borrow().player.is_playing() {
        refresh_start(ui);
        refresh_macro_buttons(ui);
        refresh_enabled(h, ui);
        invalidate_pill(h, ui);
        playing_status(h, ui);
    } else if was_running && ui.borrow().runner.is_running() {
        refresh_start(ui);
        invalidate_pill(h, ui);
        running_status(h, ui);
    }
}

/// Jendela bisa dibuka di monitor yang skalanya berbeda dari monitor utama, dan Windows tidak mengirim
/// WM_DPICHANGED untuk kondisi awal itu. Karena itu DPI disinkronkan saat WM_CREATE, sebelum kontrol dibuat.
unsafe fn sync_dpi(h: HWND, ui: &UiCell) {
    let d = GetDpiForWindow(h);
    let old = dpi();
    if d == 0 || d == old {
        return;
    }
    let mut rc: RECT = std::mem::zeroed();
    GetWindowRect(h, &mut rc);
    let k = |v: i32| v * d as i32 / old as i32;
    let scaled = RECT {
        left: rc.left,
        top: rc.top,
        right: rc.left + k(rc.right - rc.left),
        bottom: rc.top + k(rc.bottom - rc.top),
    };
    apply_dpi(h, ui, d, scaled);
}

/// Prosedur jendela utama. Semua pesan dari kontrol dan timer diteruskan ke fungsi di modul ini.
pub(super) unsafe extern "system" fn wnd_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            let cs = l as *const CreateStructW;
            SetWindowLongPtrW(h, GWLP_USERDATA, (*cs).lp_create_params as isize);
            if let Some(ui) = ui_of(h) {
                sync_dpi(h, &ui);
            }
            0
        }
        WM_ERASEBKGND => 1,
        WM_PAINT => {
            if let Some(ui) = ui_of(h) {
                if let Ok(u) = ui.try_borrow() {
                    paint(h, &u);
                    return 0;
                }
            }
            DefWindowProcW(h, msg, w, l)
        }
        // Edit yang aktif dan tidak aktif diwarnai dari tema, bukan dari warna sistem.
        WM_CTLCOLOREDIT => {
            if let Some(ui) = ui_of(h) {
                if let Ok(u) = ui.try_borrow() {
                    let p = crate::theme::pal();
                    SetTextColor(w as HDC, p.text);
                    SetBkColor(w as HDC, p.field);
                    return u.theme.field as LRESULT;
                }
            }
            DefWindowProcW(h, msg, w, l)
        }
        WM_CTLCOLORSTATIC => {
            if let Some(ui) = ui_of(h) {
                if let Ok(u) = ui.try_borrow() {
                    let p = crate::theme::pal();
                    SetTextColor(w as HDC, p.disabled);
                    SetBkColor(w as HDC, p.field_off);
                    return u.theme.field_off as LRESULT;
                }
            }
            DefWindowProcW(h, msg, w, l)
        }
        WM_DRAWITEM => {
            if let Some(ui) = ui_of(h) {
                if let Ok(u) = ui.try_borrow() {
                    draw_item(&u, &*(l as *const DrawItemStruct));
                }
            }
            1
        }
        WM_COMMAND => {
            if let Some(ui) = ui_of(h) {
                command(
                    h,
                    &ui,
                    (w & 0xFFFF) as i32,
                    ((w >> 16) & 0xFFFF) as u32,
                    l as HWND,
                );
            }
            0
        }
        WM_HOTKEY => {
            if let Some(ui) = ui_of(h) {
                match w as i32 {
                    HK_START => {
                        let (rec, play) = {
                            let u = ui.borrow();
                            (u.recording, u.player.is_playing() || u.was_playing)
                        };
                        if rec {
                            finish_record(h, &ui);
                        } else if play {
                            finish_play(h, &ui, End::User);
                        } else {
                            toggle_run(h, &ui);
                        }
                    }
                    HK_PICK => {
                        let (x, y) = cursor_pos();
                        if ui.borrow().picking {
                            end_pick(h, &ui);
                        }
                        set_pos(h, &ui, x, y);
                    }
                    HK_REC => toggle_record(h, &ui),
                    HK_PLAY => toggle_play(h, &ui),
                    HK_ESC => {
                        let (rec, play) = {
                            let u = ui.borrow();
                            (u.recording, u.player.is_playing() || u.was_playing)
                        };
                        if rec {
                            finish_record(h, &ui);
                        } else if play {
                            finish_play(h, &ui, End::User);
                        } else {
                            cancel_pick(h, &ui);
                        }
                    }
                    _ => {}
                }
            }
            0
        }
        WM_TIMER => {
            if let Some(ui) = ui_of(h) {
                tick(h, &ui);
            }
            0
        }
        picker::WM_PICKED => {
            if let Some(ui) = ui_of(h) {
                end_pick(h, &ui);
                set_pos(h, &ui, w as isize as i32, l as i32);
            }
            0
        }
        picker::WM_PICK_CANCEL => {
            if let Some(ui) = ui_of(h) {
                cancel_pick(h, &ui);
            }
            0
        }
        // Minimize ke tray bila opsi aktif; jika tidak, jendela diminimize biasa.
        WM_SYSCOMMAND => {
            if (w & SC_MASK) == SC_MINIMIZE {
                if let Some(ui) = ui_of(h) {
                    if ui.borrow().tray {
                        to_tray(h, &ui);
                        return 0;
                    }
                }
            }
            DefWindowProcW(h, msg, w, l)
        }
        shell::WM_TRAY => {
            if let Some(ui) = ui_of(h) {
                on_tray_message(h, &ui, l);
            }
            0
        }
        WM_RESTORE_APP => {
            if let Some(ui) = ui_of(h) {
                restore(h, &ui);
            }
            0
        }
        WM_DPICHANGED => {
            // LOWORD(wParam) berisi DPI X baru; lParam berisi persegi yang disarankan Windows untuk monitor baru.
            if let Some(ui) = ui_of(h) {
                let d = (w & 0xFFFF) as u32;
                if d != 0 && d != dpi() {
                    rebuild_for_dpi(h, &ui, d, *(l as *const RECT));
                }
            }
            0
        }
        WM_SIZE => {
            // Minimize: buang halaman memori yang tidak terpakai dari working set.
            if w == SIZE_MINIMIZED {
                K32EmptyWorkingSet(GetCurrentProcess());
            }
            DefWindowProcW(h, msg, w, l)
        }
        WM_DESTROY => {
            if let Some(ui) = ui_of(h) {
                shutdown(h, &ui);
            }
            PostQuitMessage(0);
            0
        }
        WM_NCDESTROY => {
            let p = GetWindowLongPtrW(h, GWLP_USERDATA) as *const RefCell<Ui>;
            SetWindowLongPtrW(h, GWLP_USERDATA, 0);
            if !p.is_null() {
                drop(Rc::from_raw(p));
            }
            DefWindowProcW(h, msg, w, l)
        }
        _ => {
            let tb = taskbar_created_msg();
            if tb != 0 && msg == tb {
                if let Some(ui) = ui_of(h) {
                    readd_tray(h, &ui);
                }
                return 0;
            }
            DefWindowProcW(h, msg, w, l)
        }
    }
}

/// Perintah dari kontrol anak (WM_COMMAND).
unsafe fn command(h: HWND, ui: &UiCell, id: i32, code: u32, ctl: HWND) {
    if code == BN_CLICKED || code == BN_DOUBLECLICKED {
        click(h, ui, id);
    } else if code == EN_CHANGE {
        match id {
            ID_HK_START | ID_HK_PICK | ID_HK_REC | ID_HK_PLAY => hotkey_edited(h, ui, id, ctl),
            ID_HOURS | ID_MINUTES | ID_SECONDS | ID_MILLIS => {
                let prev_warn = ui.borrow().rate_warn;
                let low = update_rate(h, ui);
                let busy = {
                    let u = ui.borrow();
                    u.runner.is_running() || u.recording || u.player.is_playing()
                };
                if !busy {
                    if let Some(msg) = low.and_then(low_interval_warning) {
                        set_status(h, ui, &msg, true);
                    } else if prev_warn {
                        let msg = ready_text(&ui.borrow());
                        set_status(h, ui, &msg, false);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Klik pada tombol atau segmen.
unsafe fn click(h: HWND, ui: &UiCell, id: i32) {
    if let Some(g) = seg_group_of(id) {
        choose(h, ui, g, (id - SEG_FIRST[g]) as usize);
        return;
    }
    match id {
        ID_HOLD => {
            let (hold_btn, msg) = {
                let mut u = ui.borrow_mut();
                u.hold = !u.hold;
                let note = next_start_note(&u);
                let msg = if u.hold {
                    format!("Hold mode on. Clicks continue until stopped.{note}")
                } else {
                    format!("Hold mode off.{note}")
                };
                (u.hold_btn, msg)
            };
            redraw(hold_btn);
            refresh_enabled(h, ui);
            // Label dan isian yang redup ikut berubah, jadi seluruh area digambar ulang.
            InvalidateRect(h, null(), 0);
            update_rate(h, ui);
            set_status(h, ui, &msg, false);
        }
        ID_MACRO_REC => toggle_record(h, ui),
        ID_MACRO_PLAY => toggle_play(h, ui),
        ID_MACRO_LOAD => load_macro(h, ui),
        ID_MACRO_SAVE => save_macro(h, ui),
        ID_MACRO_CLEAR => clear_macro(h, ui),
        ID_MACRO_LOOP => {
            let (btn, msg) = {
                let mut u = ui.borrow_mut();
                u.macro_loop = !u.macro_loop;
                let note = next_start_note(&u);
                let msg = if u.macro_loop {
                    format!("Macro loop on. Playback repeats until stopped.{note}")
                } else {
                    format!("Macro loop off. Playback runs once.{note}")
                };
                (u.macro_loop_btn, msg)
            };
            redraw(btn);
            redraw_macro_info(h, ui);
            set_status(h, ui, &msg, false);
        }
        ID_FOCUS => {
            let (btn, msg) = {
                let mut u = ui.borrow_mut();
                u.focus_guard = !u.focus_guard;
                let note = next_start_note(&u);
                let msg = if u.focus_guard {
                    format!("Focus guard on. Clicks pause when another window is focused.{note}")
                } else {
                    format!("Focus guard off.{note}")
                };
                (u.focus_btn, msg)
            };
            redraw(btn);
            set_status(h, ui, &msg, false);
        }
        ID_FAILSAFE => {
            let (btn, msg) = {
                let mut u = ui.borrow_mut();
                u.failsafe = !u.failsafe;
                let note = next_start_note(&u);
                let msg = if u.failsafe {
                    format!("Failsafe on. Shake the mouse or move it to the top-left corner to stop.{note}")
                } else {
                    format!("Failsafe off.{note}")
                };
                (u.failsafe_btn, msg)
            };
            redraw(btn);
            set_status(h, ui, &msg, false);
        }
        ID_TRAYOPT => {
            let (btn, tray) = {
                let mut u = ui.borrow_mut();
                u.tray = !u.tray;
                (u.tray_btn, u.tray)
            };
            redraw(btn);
            let msg = if tray {
                "Minimize now hides ClickLoop to the notification area."
            } else {
                "Minimize now uses the taskbar."
            };
            set_status(h, ui, msg, false);
        }
        ID_PICK => {
            if ui.borrow().picking {
                cancel_pick(h, ui);
            } else {
                start_pick(h, ui);
            }
        }
        ID_CLEAR => clear_points(h, ui),
        ID_START => primary_action(h, ui),
        ID_PIN => {
            let (btn, topmost) = {
                let mut u = ui.borrow_mut();
                u.topmost = !u.topmost;
                (u.pin_btn, u.topmost)
            };
            set_topmost(h, topmost);
            redraw(btn);
            let msg = if topmost {
                "Window stays on top of other windows."
            } else {
                "Window no longer stays on top."
            };
            set_status(h, ui, msg, false);
        }
        ID_MINI => {
            let next = !ui.borrow().mini;
            set_mode(h, ui, next);
        }
        ID_EXPAND => set_mode(h, ui, false),
        ID_THEME => toggle_theme(h, ui),
        ID_LOAD => load_profile(h, ui),
        ID_SAVE => save_profile(h, ui),
        IDCANCEL => {
            let (rec, play) = {
                let u = ui.borrow();
                (u.recording, u.player.is_playing() || u.was_playing)
            };
            if rec {
                finish_record(h, ui);
            } else if play {
                finish_play(h, ui, End::User);
            } else {
                cancel_pick(h, ui);
            }
        }
        _ => {}
    }
}

/// Memilih satu opsi di grup segmen `g` (urutan SEG_GROUPS).
unsafe fn choose(h: HWND, ui: &UiCell, g: usize, k: usize) {
    if g == 4 {
        set_tab(h, ui, k);
        return;
    }
    {
        let mut u = ui.borrow_mut();
        match g {
            0 => u.button = k as u32,
            1 => u.kind = k as u32,
            2 => u.trig = k as u32,
            _ => u.pos = k as u32,
        }
    }
    match g {
        1 => {
            update_rate(h, ui);
        }
        2 => redraw_rect(h, TRIG_HINT),
        3 => {
            refresh_enabled(h, ui);
            redraw_rect(h, POS_HINT);
        }
        _ => {}
    }
    redraw_group(ui, g);
}

/// Dipanggil dari loop pesan setiap kali fokus keyboard berpindah.
pub(super) unsafe fn focus_changed(h: HWND, ui: &UiCell, old: HWND, new: HWND) {
    let (rects, pause_action, ctls) = {
        let mut u = ui.borrow_mut();
        u.focus = new;
        let mut rects = Vec::new();
        if !u.mini {
            for i in 0..FIELDS.len() {
                let c = u.ctl[i];
                if !c.is_null() && (c == old || c == new) && field_visible(u.tab, i) {
                    rects.push(field_rect_for(u.tab, i));
                }
            }
        }
        let hk_ctls = [
            u.ctl[F_HK_START],
            u.ctl[F_HK_PICK],
            u.ctl[F_HK_REC],
            u.ctl[F_HK_PLAY],
        ];
        let hk = |x: HWND| !x.is_null() && hk_ctls.contains(&x);
        let (into, out_of) = (hk(new), hk(old));
        let action = if into && !u.hk_paused {
            u.hk_paused = true;
            1 // pause
        } else if !into && u.hk_paused {
            u.hk_paused = false;
            if out_of {
                2
            } else {
                3
            } // resume
        } else {
            0
        };
        (rects, action, hk_ctls)
    };
    for rc in rects {
        InvalidateRect(h, &rc, 0);
    }
    if pause_action == 1 {
        UnregisterHotKey(h, HK_START);
        UnregisterHotKey(h, HK_PICK);
        UnregisterHotKey(h, HK_REC);
        UnregisterHotKey(h, HK_PLAY);
        set_status(
            h,
            ui,
            "Press the new key combination. Hotkeys are paused while you type.",
            false,
        );
    } else if pause_action == 2 || pause_action == 3 {
        let out_of = pause_action == 2;
        for (c, id) in [
            (ctls[0], ID_HK_START),
            (ctls[1], ID_HK_PICK),
            (ctls[2], ID_HK_REC),
            (ctls[3], ID_HK_PLAY),
        ] {
            hotkey_edited(h, ui, id, c);
            let key = {
                let u = ui.borrow();
                match id {
                    ID_HK_START => u.start_key,
                    ID_HK_PICK => u.pick_key,
                    ID_HK_REC => u.rec_key,
                    _ => u.play_key,
                }
            };
            SendMessageW(c, HKM_SETHOTKEY, keys::to_word(key), 0);
        }
        match apply_hotkeys(h, ui) {
            Some(e) => set_status(h, ui, &e, true),
            None if out_of && !ui.borrow().warn => {
                let msg = ready_text(&ui.borrow());
                set_status(h, ui, &msg, false);
            }
            None => {}
        }
    }
}

/// Validasi isi kotak hotkey. Kombinasi yang tidak sah dikembalikan ke hotkey sebelumnya.
unsafe fn hotkey_edited(h: HWND, ui: &UiCell, id: i32, ctl: HWND) {
    let k = keys::from_word(SendMessageW(ctl, HKM_GETHOTKEY, 0, 0));
    if k.vk == 0 {
        return; // modifier saja, atau dikosongkan: tunggu tombol utama
    }
    let (old, others) = {
        let u = ui.borrow();
        match id {
            ID_HK_START => (u.start_key, [u.pick_key, u.rec_key, u.play_key]),
            ID_HK_PICK => (u.pick_key, [u.start_key, u.rec_key, u.play_key]),
            ID_HK_REC => (u.rec_key, [u.start_key, u.pick_key, u.play_key]),
            _ => (u.play_key, [u.start_key, u.pick_key, u.rec_key]),
        }
    };
    if k == old {
        return;
    }
    let err = match keys::check(k) {
        Err(e) => Some(e),
        Ok(()) if others.contains(&k) => Some("Each hotkey must be different."),
        Ok(()) => None,
    };
    if let Some(e) = err {
        SendMessageW(ctl, HKM_SETHOTKEY, keys::to_word(old), 0);
        set_status(h, ui, e, true);
        return;
    }
    {
        let mut u = ui.borrow_mut();
        match id {
            ID_HK_START => u.start_key = k,
            ID_HK_PICK => u.pick_key = k,
            ID_HK_REC => u.rec_key = k,
            _ => u.play_key = k,
        }
    }
    match id {
        ID_HK_START => refresh_start(ui),
        ID_HK_REC | ID_HK_PLAY => {
            refresh_macro_buttons(ui);
            refresh_start(ui);
        }
        _ => {}
    }
    let what = match id {
        ID_HK_START => "Start / stop",
        ID_HK_PICK => "Pick position",
        ID_HK_REC => "Record macro",
        _ => "Play macro",
    };
    set_status(h, ui, &format!("{what} hotkey: {}", keys::name(k)), false);
}
