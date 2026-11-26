//! Pembuatan kontrol anak dan subclass tombol. Urutan pembuatan sama dengan urutan Tab.
use std::ptr::{null, null_mut};

use crate::config::{Hotkey, Settings};
use crate::keys;
use crate::theme as th;
use crate::win32::*;

use super::actions::{
    apply_hotkeys, apply_titlebar, apply_view, ready_text, refresh_macro_buttons, refresh_start,
    set_status, set_topmost,
};
use super::layout::*;
use super::settings::{field_text, low_interval_warning, update_rate};
use super::window::{ui_of, UiCell};

/// Membuat kontrol anak dengan font dari tema.
unsafe fn child(
    parent: HWND,
    class: &str,
    text: &str,
    style: u32,
    id: i32,
    rc: RECT,
    font: HGDIOBJ,
) -> HWND {
    let c = wide(class);
    let t = wide(text);
    let hwnd = CreateWindowExW(
        0,
        c.as_ptr(),
        t.as_ptr(),
        WS_CHILD | WS_VISIBLE | style,
        rc.left,
        rc.top,
        rc.right - rc.left,
        rc.bottom - rc.top,
        parent,
        id as usize as HMENU,
        GetModuleHandleW(null()),
        null_mut(),
    );
    if !font.is_null() {
        SendMessageW(hwnd, WM_SETFONT, font as usize, 0);
    }
    hwnd
}

/// Persegi dalam (piksel fisik) untuk kontrol EDIT `i` pada `tab`.
pub(super) unsafe fn edit_inner_rect(small: HGDIOBJ, tab: usize, i: usize) -> RECT {
    let f = &FIELDS[i];
    let rc = field_rect_for(tab, i);
    let pad_l = if f.prefix.is_empty() {
        sc(4)
    } else {
        sc(8) + th::measure(small, f.prefix) + sc(6)
    };
    let pad_r = sc(8)
        + if f.unit.is_empty() {
            0
        } else {
            th::measure(small, f.unit) + sc(4)
        };
    let eh = sc(EDIT_H);
    let top = rc.top + (rc.bottom - rc.top - eh) / 2;
    RECT {
        left: rc.left + pad_l,
        top,
        right: rc.right - pad_r,
        bottom: top + eh,
    }
}

/// Persegi dalam (piksel fisik) untuk kontrol hotkey `i` pada `tab`.
pub(super) fn hotkey_inner_rect(tab: usize, i: usize) -> RECT {
    let rc = field_rect_for(tab, i);
    let eh = sc(EDIT_H);
    let top = rc.top + (rc.bottom - rc.top - eh) / 2;
    RECT {
        left: rc.left + sc(8),
        top,
        right: rc.right - sc(8),
        bottom: top + eh,
    }
}

/// EDIT tanpa border di dalam kotak isian yang dilukis induk.
unsafe fn edit(h: HWND, body: HGDIOBJ, small: HGDIOBJ, i: usize, value: &str) -> HWND {
    let f = &FIELDS[i];
    let inner = edit_inner_rect(small, TAB_CLICKER, i);
    let mut style = WS_TABSTOP | ES_AUTOHSCROLL;
    if f.numeric {
        style |= ES_NUMBER;
    }
    if !f.unit.is_empty() {
        style |= ES_RIGHT;
    }
    let c = child(h, "EDIT", value, style, f.id, inner, body);
    SendMessageW(c, EM_LIMITTEXT, f.max_len, 0);
    SendMessageW(c, EM_SETMARGINS, EC_LEFTMARGIN | EC_RIGHTMARGIN, 0);
    c
}

/// Kotak hotkey (msctls_hotkey32) di dalam kotak isian yang dilukis induk.
unsafe fn hotkey_box(h: HWND, body: HGDIOBJ, tab: usize, i: usize, key: Hotkey) -> HWND {
    let inner = hotkey_inner_rect(tab, i);
    let c = child(
        h,
        "msctls_hotkey32",
        "",
        WS_TABSTOP,
        FIELDS[i].id,
        inner,
        body,
    );
    let ex = GetWindowLongPtrW(c, GWL_EXSTYLE);
    SetWindowLongPtrW(c, GWL_EXSTYLE, ex & !WS_EX_CLIENTEDGE);
    SetWindowPos(
        c,
        null_mut(),
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
    );
    SendMessageW(c, HKM_SETHOTKEY, keys::to_word(key), 0);
    SetWindowSubclass(c, hotkey_proc, 2, h as usize);
    c
}

/// Kotak hotkey dilukis sendiri karena kontrol asli selalu berlatar putih.
unsafe extern "system" fn hotkey_proc(
    c: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
    _id: usize,
    parent: usize,
) -> LRESULT {
    match msg {
        WM_ERASEBKGND => return 1,
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let dc = BeginPaint(c, &mut ps);
            if !dc.is_null() {
                let mut rc: RECT = std::mem::zeroed();
                GetClientRect(c, &mut rc);
                let p = th::pal();
                th::fill(dc, rc, p.field);
                let word = SendMessageW(c, HKM_GETHOTKEY, 0, 0);
                let (text, color) = if word & 0xFF == 0 {
                    ("Not set".to_string(), p.muted)
                } else {
                    (keys::name(keys::from_word(word)), p.text)
                };
                let font = ui_of(parent as HWND)
                    .and_then(|u| u.try_borrow().ok().map(|b| b.theme.body))
                    .unwrap_or(null_mut());
                th::text(dc, &text, rc, font, color, DT_CENTER);
            }
            EndPaint(c, &ps);
            return 0;
        }
        WM_NCDESTROY => {
            RemoveWindowSubclass(c, hotkey_proc, 2);
        }
        _ => {}
    }
    DefSubclassProc(c, msg, w, l)
}

/// Tombol owner-draw dengan subclass untuk efek hover.
unsafe fn odbutton(h: HWND, id: i32, rc: RECT, text: &str) -> HWND {
    let c = child(
        h,
        "BUTTON",
        text,
        BS_OWNERDRAW | WS_TABSTOP,
        id,
        rc,
        null_mut(),
    );
    SetWindowSubclass(c, button_proc, 1, h as usize);
    c
}

/// Satu grup segmen (lihat SEG_GROUPS): tombol berderet rata dalam wadah yang dilukis induk.
unsafe fn segment(h: HWND, segs: &mut [HWND; 14], g: usize) {
    let (gx, gy, gw, gh) = SEG_GROUPS[g];
    let names = seg_names(g);
    let seg_w = (gw - 6) / names.len() as i32;
    for (k, name) in names.iter().enumerate() {
        let rc = r(gx + 3 + seg_w * k as i32, gy + 3, seg_w, gh - 6);
        segs[SEG_BASE[g] + k] = odbutton(h, SEG_FIRST[g] + k as i32, rc, name);
    }
}

/// Membuat semua kontrol jendela utama dan menerapkan keadaan awal.
pub(super) unsafe fn build(h: HWND, ui: &UiCell, s: &Settings) {
    let (body, small, dark, topmost) = {
        let u = ui.borrow();
        (u.theme.body, u.theme.small, u.dark, u.topmost)
    };
    let mut ctl = [null_mut(); 18];
    let mut segs = [null_mut(); 14];

    // Header: Tab switcher
    segment(h, &mut segs, 4);

    // Tab 0 (Auto Clicker) + kontrol bersama
    for i in [0, 1, 2, 3, 4, 5] {
        ctl[i] = edit(h, body, small, i, &field_text(s, i));
    }
    segment(h, &mut segs, 0);
    segment(h, &mut segs, 1);
    let hold_btn = odbutton(h, ID_HOLD, rt(HOLD_RC), "Hold button down");
    for i in [F_DELAY_MIN, F_DELAY_SEC, F_DELAY_MS] {
        ctl[i] = edit(h, body, small, i, &field_text(s, i));
    }
    ctl[F_HK_START] = hotkey_box(h, body, TAB_CLICKER, F_HK_START, s.start_key);
    ctl[F_HK_PICK] = hotkey_box(h, body, TAB_CLICKER, F_HK_PICK, s.pick_key);
    for i in [6, 7] {
        ctl[i] = edit(h, body, small, i, &field_text(s, i));
    }
    segment(h, &mut segs, 3);
    for i in [F_X, F_Y, F_POS_JITTER] {
        ctl[i] = edit(h, body, small, i, &field_text(s, i));
    }
    let pick_btn = odbutton(h, ID_PICK, rt(PICK_RC), "Pick");
    let clear_btn = odbutton(h, ID_CLEAR, rt(CLEAR_RC), "Clear points");
    segment(h, &mut segs, 2);

    // Tab 1 (Record & Play)
    let macro_rec_btn = odbutton(h, ID_MACRO_REC, rt(MACRO_REC_RC), "Record");
    let macro_play_btn = odbutton(h, ID_MACRO_PLAY, rt(MACRO_PLAY_RC), "Play");
    let macro_load_btn = odbutton(h, ID_MACRO_LOAD, rt(MACRO_LOAD_RC), "Load");
    let macro_save_btn = odbutton(h, ID_MACRO_SAVE, rt(MACRO_SAVE_RC), "Save");
    let macro_clear_btn = odbutton(h, ID_MACRO_CLEAR, rt(MACRO_CLEAR_RC), "Clear");
    let macro_loop_btn = odbutton(h, ID_MACRO_LOOP, rt(MACRO_LOOP_RC), "Loop playback");
    ctl[F_HK_REC] = hotkey_box(h, body, TAB_MACRO, F_HK_REC, s.rec_key);
    ctl[F_HK_PLAY] = hotkey_box(h, body, TAB_MACRO, F_HK_PLAY, s.play_key);

    // Kontrol bersama (Safety & Profiles)
    let focus_btn = odbutton(h, ID_FOCUS, rt(FOCUS_RC_0), "Focus guard");
    let failsafe_btn = odbutton(h, ID_FAILSAFE, rt(FAILSAFE_RC_0), "Failsafe");
    let tray_btn = odbutton(h, ID_TRAYOPT, rt(TRAY_RC_0), "Minimize to tray");
    let load_btn = odbutton(h, ID_LOAD, rt(LOAD_RC_0), "Load profile");
    let save_btn = odbutton(h, ID_SAVE, rt(SAVE_RC_0), "Save profile");

    // Footer & header
    let start_btn = odbutton(h, ID_START, rt(START_RC), "Start");
    let mini_btn = odbutton(h, ID_MINI, rt(MINI_BTN_RC), "Mini mode");
    let pin_btn = odbutton(h, ID_PIN, rt(PIN_RC), "Keep on top");
    let theme_btn = odbutton(h, ID_THEME, rt(THEME_RC), "Switch theme");
    let expand_btn = odbutton(h, ID_EXPAND, rt(MINI_EXPAND), "Expand");

    {
        let mut u = ui.borrow_mut();
        u.ctl = ctl;
        u.segs = segs;
        u.hold_btn = hold_btn;
        u.focus_btn = focus_btn;
        u.failsafe_btn = failsafe_btn;
        u.tray_btn = tray_btn;
        u.pick_btn = pick_btn;
        u.clear_btn = clear_btn;
        u.load_btn = load_btn;
        u.save_btn = save_btn;
        u.start_btn = start_btn;
        u.pin_btn = pin_btn;
        u.mini_btn = mini_btn;
        u.expand_btn = expand_btn;
        u.theme_btn = theme_btn;
        u.macro_rec_btn = macro_rec_btn;
        u.macro_play_btn = macro_play_btn;
        u.macro_load_btn = macro_load_btn;
        u.macro_save_btn = macro_save_btn;
        u.macro_clear_btn = macro_clear_btn;
        u.macro_loop_btn = macro_loop_btn;
    }

    apply_titlebar(h, dark);
    if topmost {
        set_topmost(h, true);
    }
    let low = update_rate(h, ui);
    apply_view(h, ui);
    refresh_start(ui);
    refresh_macro_buttons(ui);
    let msg = match apply_hotkeys(h, ui) {
        Some(e) => (e, true),
        None => match low.and_then(low_interval_warning) {
            Some(w) => (w, true),
            None => (ready_text(&ui.borrow()), false),
        },
    };
    set_status(h, ui, &msg.0, msg.1);
    SendMessageW(h, WM_CHANGEUISTATE, UIS_SET | (UISF_HIDEFOCUS << 16), 0);
}

unsafe extern "system" fn button_proc(
    b: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
    _id: usize,
    parent: usize,
) -> LRESULT {
    match msg {
        WM_MOUSEMOVE => {
            let changed = ui_of(parent as HWND).and_then(|ui| {
                let mut u = ui.try_borrow_mut().ok()?;
                if u.hot != b {
                    let old = u.hot;
                    u.hot = b;
                    Some(old)
                } else {
                    None
                }
            });
            if let Some(old) = changed {
                if !old.is_null() {
                    InvalidateRect(old, null(), 0);
                }
                InvalidateRect(b, null(), 0);
                let mut t = TRACKMOUSEEVENT {
                    cb_size: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                    flags: TME_LEAVE,
                    hwnd: b,
                    hover_time: 0,
                };
                TrackMouseEvent(&mut t);
            }
        }
        WM_MOUSELEAVE => {
            let cleared = ui_of(parent as HWND)
                .and_then(|ui| {
                    let mut u = ui.try_borrow_mut().ok()?;
                    if u.hot == b {
                        u.hot = null_mut();
                        Some(true)
                    } else {
                        Some(false)
                    }
                })
                .unwrap_or(false);
            if cleared {
                InvalidateRect(b, null(), 0);
            }
        }
        WM_ERASEBKGND => return 1,
        WM_NCDESTROY => {
            RemoveWindowSubclass(b, button_proc, 1);
        }
        _ => {}
    }
    DefSubclassProc(b, msg, w, l)
}
