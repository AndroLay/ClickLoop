//! Geometri dan ID kontrol jendela utama, dalam piksel logis (96 DPI). `r()` mengalikan dengan DPI.
//! Desain kompak 2-tab (`Auto Clicker` | `Record & Play`) berukuran 580x540:
//! kolom kiri (x=14, w=268) dan kanan (x=298, w=268) sama-sama 50..492 (0 px selisih).
use crate::win32::*;

pub const W: i32 = 580;
pub const H: i32 = 540;
pub const MINI_W: i32 = 320;
pub const MINI_H: i32 = 140;
pub const FIELD_H: i32 = 26;
pub const EDIT_H: i32 = 18;

pub const TAB_CLICKER: usize = 0;
pub const TAB_MACRO: usize = 1;

pub const TIMER_UI: usize = 1;
pub const TIMER_MS: u32 = 250;
pub const HK_START: i32 = 1;
pub const HK_PICK: i32 = 2;
pub const HK_ESC: i32 = 3;
pub const HK_REC: i32 = 4;
pub const HK_PLAY: i32 = 5;

pub const ID_HOURS: i32 = 101;
pub const ID_MINUTES: i32 = 102;
pub const ID_SECONDS: i32 = 103;
pub const ID_MILLIS: i32 = 104;
pub const ID_JITTER: i32 = 105;
pub const ID_PRESS: i32 = 106;
pub const ID_MAX_CLICKS: i32 = 107;
pub const ID_MAX_SECS: i32 = 108;
pub const ID_X: i32 = 109;
pub const ID_Y: i32 = 110;
pub const ID_HK_START: i32 = 111;
pub const ID_HK_PICK: i32 = 112;
pub const ID_POS_JITTER: i32 = 113;
pub const ID_DELAY_MIN: i32 = 114;
pub const ID_DELAY_SEC: i32 = 115;
pub const ID_DELAY_MS: i32 = 116;
pub const ID_HK_REC: i32 = 117;
pub const ID_HK_PLAY: i32 = 118;
/// Segmen: ID awal per grup (lihat SEG_FIRST). Setiap grup memakai ID berurutan.
pub const ID_BTN: i32 = 120;
pub const ID_KIND: i32 = 123;
pub const ID_TRIG: i32 = 150;
pub const ID_POS: i32 = 160;
pub const ID_TAB: i32 = 170;
pub const ID_HOLD: i32 = 130;
pub const ID_PICK: i32 = 140;
pub const ID_START: i32 = 141;
pub const ID_PIN: i32 = 142;
pub const ID_FOCUS: i32 = 180;
pub const ID_FAILSAFE: i32 = 181;
pub const ID_CLEAR: i32 = 190;
pub const ID_LOAD: i32 = 191;
pub const ID_SAVE: i32 = 192;
pub const ID_TRAYOPT: i32 = 193;
pub const ID_MINI: i32 = 194;
pub const ID_EXPAND: i32 = 195;
pub const ID_THEME: i32 = 196;
pub const ID_MACRO_REC: i32 = 200;
pub const ID_MACRO_PLAY: i32 = 201;
pub const ID_MACRO_LOAD: i32 = 202;
pub const ID_MACRO_SAVE: i32 = 203;
pub const ID_MACRO_CLEAR: i32 = 204;
pub const ID_MACRO_LOOP: i32 = 205;

/// Indeks di FIELDS dan Ui::ctl.
pub const F_MAX_SECS: usize = 7;
pub const F_X: usize = 8;
pub const F_Y: usize = 9;
pub const F_HK_START: usize = 10;
pub const F_HK_PICK: usize = 11;
pub const F_POS_JITTER: usize = 12;
pub const F_DELAY_MIN: usize = 13;
pub const F_DELAY_SEC: usize = 14;
pub const F_DELAY_MS: usize = 15;
pub const F_HK_REC: usize = 16;
pub const F_HK_PLAY: usize = 17;
/// Isian yang tidak berlaku dalam mode hold.
pub const HOLD_OFF: [usize; 7] = [0, 1, 2, 3, 4, 5, 6];

/// Kotak isian: posisi default (Tab 0 atau Tab 1), lebar, satuan di kiri/kanan, dan panjang maksimum.
pub struct Field {
    pub id: i32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub prefix: &'static str,
    pub unit: &'static str,
    pub max_len: usize,
    pub numeric: bool,
}

#[allow(clippy::too_many_arguments)]
const fn fd(
    id: i32,
    x: i32,
    y: i32,
    w: i32,
    prefix: &'static str,
    unit: &'static str,
    max_len: usize,
    numeric: bool,
) -> Field {
    Field {
        id,
        x,
        y,
        w,
        prefix,
        unit,
        max_len,
        numeric,
    }
}

pub const FIELDS: [Field; 18] = [
    fd(ID_HOURS, 26, 82, 50, "", "h", 2, true),
    fd(ID_MINUTES, 82, 82, 50, "", "m", 2, true),
    fd(ID_SECONDS, 138, 82, 50, "", "s", 2, true),
    fd(ID_MILLIS, 194, 82, 76, "", "ms", 4, true),
    fd(ID_JITTER, 26, 132, 119, "\u{B1}", "ms", 4, true),
    fd(ID_PRESS, 151, 132, 119, "", "ms", 5, true),
    fd(ID_MAX_CLICKS, 310, 98, 119, "", "clicks", 10, true),
    fd(ID_MAX_SECS, 435, 98, 119, "", "sec", 10, true),
    fd(ID_X, 310, 206, 76, "X", "", 6, false),
    fd(ID_Y, 392, 206, 76, "Y", "", 6, false),
    fd(ID_HK_START, 26, 446, 119, "", "", 0, false),
    fd(ID_HK_PICK, 151, 446, 119, "", "", 0, false),
    fd(ID_POS_JITTER, 310, 238, 119, "\u{B1}", "px", 3, true),
    fd(ID_DELAY_MIN, 26, 388, 74, "", "m", 2, true),
    fd(ID_DELAY_SEC, 106, 388, 74, "", "s", 2, true),
    fd(ID_DELAY_MS, 186, 388, 84, "", "ms", 4, true),
    fd(ID_HK_REC, 26, 438, 119, "", "", 0, false),
    fd(ID_HK_PLAY, 151, 438, 119, "", "", 0, false),
];

/// Apakah field `i` ditampilkan pada `tab`.
pub fn field_visible(tab: usize, i: usize) -> bool {
    if tab == TAB_CLICKER {
        i <= F_DELAY_MS
    } else {
        matches!(
            i,
            F_MAX_SECS | F_DELAY_MIN | F_DELAY_SEC | F_DELAY_MS | F_HK_REC | F_HK_PLAY
        )
    }
}

/// Kotak logis `(x, y, w, h)` untuk field `i` sesuai `tab` aktif (single source of truth).
pub fn field_box(tab: usize, i: usize) -> (i32, i32, i32, i32) {
    if tab == TAB_MACRO {
        match i {
            F_MAX_SECS => (310, 98, 244, FIELD_H),
            F_DELAY_MIN => (26, 316, 74, FIELD_H),
            F_DELAY_SEC => (106, 316, 74, FIELD_H),
            F_DELAY_MS => (186, 316, 84, FIELD_H),
            _ => {
                let f = &FIELDS[i];
                (f.x, f.y, f.w, FIELD_H)
            }
        }
    } else {
        let f = &FIELDS[i];
        (f.x, f.y, f.w, FIELD_H)
    }
}

pub fn field_rect_for(tab: usize, i: usize) -> RECT {
    rt(field_box(tab, i))
}

/// Kartu Tab 0 (Auto Clicker): kiri & kanan sama-sama 50..492 (0 px selisih).
pub const CARDS_CLICKER: [(i32, i32, i32, i32, &str); 7] = [
    // Kolom kiri (x=14, w=268): 50..174, 182..348, 356..492
    (14, 50, 268, 124, "Click interval"),
    (14, 182, 268, 166, "Click options"),
    (14, 356, 268, 136, "Delayed start"),
    // Kolom kanan (x=298, w=268): 50..134, 142..276, 284..364, 372..492
    (298, 50, 268, 84, "Stop after"),
    (298, 142, 268, 134, "Cursor position"),
    (298, 284, 268, 80, "Trigger"),
    (298, 372, 268, 120, ""),
];

/// Kartu Tab 1 (Record & Play): baris kartu kiri & kanan sejajar 50..276, 284..380, 388..492.
pub const CARDS_MACRO: [(i32, i32, i32, i32, &str); 7] = [
    // Kolom kiri (x=14, w=268): 50..276, 284..380, 388..492
    (14, 50, 268, 226, "Record & playback"),
    (14, 284, 268, 96, "Delayed start"),
    (14, 388, 268, 104, "Macro hotkeys"),
    // Kolom kanan (x=298, w=268): 50..142, 150..276, 284..380, 388..492
    (298, 50, 268, 92, "Stop after"),
    (298, 150, 268, 126, "Safety & window"),
    (298, 284, 268, 96, "Profiles"),
    (298, 388, 268, 104, "Macro details"),
];

/// Label kecil Tab 0: teks, x, y, lebar, tinggi, redup dalam mode hold.
pub const LABELS_CLICKER: [(&str, i32, i32, i32, i32, bool); 8] = [
    ("Random offset", 26, 114, 119, 15, true),
    ("Press duration", 151, 114, 119, 15, true),
    ("Mouse button", 26, 212, 244, 15, false),
    ("Click type", 26, 260, 244, 15, true),
    ("Clicks", 310, 80, 119, 15, true),
    ("Time", 435, 80, 119, 15, false),
    ("Start / stop", 26, 426, 119, 16, false),
    ("Pick position", 151, 426, 119, 16, false),
];

/// Label kecil Tab 1: teks, x, y, lebar, tinggi.
pub const LABELS_MACRO: [(&str, i32, i32, i32, i32); 8] = [
    ("Captures mouse, scroll & keyboard", 26, 82, 244, 16),
    ("Press Esc or hotkey anytime to stop", 26, 236, 244, 16),
    ("Countdown before playback starts", 26, 350, 244, 16),
    ("Record / stop", 26, 420, 119, 16),
    ("Play / stop", 151, 420, 119, 16),
    ("Work globally from any window", 26, 468, 244, 15),
    ("Time (for looped playback)", 310, 80, 244, 15),
    ("Saves settings & hotkeys (.ini)", 310, 354, 244, 16),
];

/// Well segmented: x, y, lebar, tinggi. Grup 0..3 di Tab 0; grup 4 = Tab switcher (header/mini).
pub const SEG_GROUPS: [(i32, i32, i32, i32); 5] = [
    (26, 228, 244, FIELD_H),
    (26, 276, 244, FIELD_H),
    (310, 314, 244, FIELD_H),
    (310, 174, 244, FIELD_H),
    (160, 11, 220, 28),
];
/// ID awal tiap grup segmen (0 button, 1 tipe klik, 2 pemicu, 3 posisi, 4 tab mode).
pub const SEG_FIRST: [i32; 5] = [ID_BTN, ID_KIND, ID_TRIG, ID_POS, ID_TAB];
/// Indeks pertama tiap grup di Ui::segs.
pub const SEG_BASE: [usize; 5] = [0, 3, 6, 9, 12];

pub const BUTTON_NAMES: [&str; 3] = ["Left", "Right", "Middle"];
const KIND_NAMES: [&str; 3] = ["Single", "Double", "Triple"];
const TRIG_NAMES: [&str; 3] = ["Hotkey", "Mouse 4", "Mouse 5"];
const POS_NAMES: [&str; 3] = ["Cursor", "Fixed", "Sequence"];
const TAB_NAMES: [&str; 2] = ["Auto Clicker", "Record & Play"];
pub const HOLD_HINT: &str = "Press and hold until stopped";
pub const MACRO_LOOP_HINT: &str = "Repeat recorded mouse & keys";

pub fn seg_names(g: usize) -> &'static [&'static str] {
    match g {
        0 => &BUTTON_NAMES,
        1 => &KIND_NAMES,
        2 => &TRIG_NAMES,
        3 => &POS_NAMES,
        _ => &TAB_NAMES,
    }
}

/// Grup segmen yang memuat ID kontrol ini.
pub fn seg_group_of(id: i32) -> Option<usize> {
    (0..5).find(|&g| {
        let f = SEG_FIRST[g];
        id >= f && ((id - f) as usize) < seg_names(g).len()
    })
}

/// Nama pemicu untuk teks status dan pil (indeks = Ui::trig).
pub fn trig_name(trig: u32) -> &'static str {
    TRIG_NAMES[(trig as usize).min(2)]
}

/// Well tab switcher di mode penuh vs mode mini.
pub const TAB_WELL_FULL: (i32, i32, i32, i32) = (160, 11, 220, 28);
pub const TAB_WELL_MINI: (i32, i32, i32, i32) = (14, 68, 292, 24);

pub fn tab_well_of(mini: bool) -> (i32, i32, i32, i32) {
    if mini {
        TAB_WELL_MINI
    } else {
        TAB_WELL_FULL
    }
}

pub const RATE: (i32, i32, i32, i32) = (116, 58, 154, 18);
pub const DELAY_HINT_CLICKER: (i32, i32, i32, i32) = (166, 364, 104, 18);
pub const DELAY_HINT_MACRO: (i32, i32, i32, i32) = (166, 292, 104, 18);
pub const MACRO_HINT: (i32, i32, i32, i32) = (142, 58, 128, 18);
pub const STOP_HINT: (i32, i32, i32, i32) = (438, 58, 116, 18);
pub const POS_HINT: (i32, i32, i32, i32) = (422, 150, 132, 18);
pub const TRIG_HINT: (i32, i32, i32, i32) = (310, 344, 244, 15);
pub const MACRO_DETAIL_1: (i32, i32, i32, i32) = (310, 420, 244, 18);
pub const MACRO_DETAIL_2: (i32, i32, i32, i32) = (310, 442, 244, 18);
pub const MACRO_DETAIL_3: (i32, i32, i32, i32) = (310, 464, 244, 18);

// Kontrol Tab 0 (Auto Clicker)
pub const HOLD_RC: (i32, i32, i32, i32) = (26, 308, 244, 32);
pub const PICK_RC: (i32, i32, i32, i32) = (474, 206, 80, FIELD_H);
pub const CLEAR_RC: (i32, i32, i32, i32) = (435, 238, 119, FIELD_H);
pub const FOCUS_RC_0: (i32, i32, i32, i32) = (310, 380, 244, 24);
pub const FAILSAFE_RC_0: (i32, i32, i32, i32) = (310, 406, 244, 24);
pub const TRAY_RC_0: (i32, i32, i32, i32) = (310, 432, 244, 24);
pub const LOAD_RC_0: (i32, i32, i32, i32) = (310, 460, 119, 24);
pub const SAVE_RC_0: (i32, i32, i32, i32) = (435, 460, 119, 24);

// Kontrol Tab 1 (Record & Play)
pub const MACRO_REC_RC: (i32, i32, i32, i32) = (26, 108, 119, 32);
pub const MACRO_PLAY_RC: (i32, i32, i32, i32) = (151, 108, 119, 32);
pub const MACRO_LOAD_RC: (i32, i32, i32, i32) = (26, 148, 77, 28);
pub const MACRO_SAVE_RC: (i32, i32, i32, i32) = (109, 148, 77, 28);
pub const MACRO_CLEAR_RC: (i32, i32, i32, i32) = (192, 148, 78, 28);
pub const MACRO_LOOP_RC: (i32, i32, i32, i32) = (26, 188, 244, 34);
pub const FOCUS_RC_1: (i32, i32, i32, i32) = (310, 184, 244, 26);
pub const FAILSAFE_RC_1: (i32, i32, i32, i32) = (310, 212, 244, 26);
pub const TRAY_RC_1: (i32, i32, i32, i32) = (310, 240, 244, 26);
pub const LOAD_RC_1: (i32, i32, i32, i32) = (310, 320, 119, 28);
pub const SAVE_RC_1: (i32, i32, i32, i32) = (435, 320, 119, 28);

// Header & Footer
pub const PILL: (i32, i32, i32, i32) = (388, 13, 108, 24);
pub const THEME_RC: (i32, i32, i32, i32) = (504, 11, 28, 28);
pub const PIN_RC: (i32, i32, i32, i32) = (538, 11, 28, 28);
pub const STATUS: (i32, i32, i32, i32) = (14, 500, 324, 30);
pub const MINI_BTN_RC: (i32, i32, i32, i32) = (346, 500, 96, 30);
pub const START_RC: (i32, i32, i32, i32) = (450, 500, 116, 30);

// Mini Mode (320x140): Pill + Mode tag + Expand + Pin, Status, Tab switcher, Tombol aksi sesuai mode.
pub const MINI_PILL: (i32, i32, i32, i32) = (14, 13, 102, 24);
pub const MINI_MODE_LBL: (i32, i32, i32, i32) = (120, 13, 78, 24);
pub const MINI_EXPAND: (i32, i32, i32, i32) = (202, 12, 74, 26);
pub const MINI_PIN: (i32, i32, i32, i32) = (280, 12, 26, 26);
pub const MINI_STATUS: (i32, i32, i32, i32) = (14, 44, 292, 18);
pub const MINI_START: (i32, i32, i32, i32) = (14, 98, 292, 32);
pub const MINI_REC: (i32, i32, i32, i32) = (14, 98, 142, 32);
pub const MINI_PLAY: (i32, i32, i32, i32) = (164, 98, 142, 32);

/// Persegi logis ke piksel fisik.
pub fn r(x: i32, y: i32, w: i32, h: i32) -> RECT {
    RECT {
        left: sc(x),
        top: sc(y),
        right: sc(x + w),
        bottom: sc(y + h),
    }
}

pub fn rt(t: (i32, i32, i32, i32)) -> RECT {
    r(t.0, t.1, t.2, t.3)
}

/// Dua persegi bersinggungan.
pub fn meets(a: &RECT, b: &RECT) -> bool {
    a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom
}

/// Pil untuk mode penuh atau mini.
pub fn pill_of(mini: bool) -> (i32, i32, i32, i32) {
    if mini {
        MINI_PILL
    } else {
        PILL
    }
}

/// Baris status untuk mode penuh atau mini.
pub fn status_of(mini: bool) -> (i32, i32, i32, i32) {
    if mini {
        MINI_STATUS
    } else {
        STATUS
    }
}
