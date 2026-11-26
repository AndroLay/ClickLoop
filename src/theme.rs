//! Tema terang gaya Fluent (Windows 11) dan primitif gambar.
//! Sudut membulat dirender sendiri dengan anti-aliasing: hanya piksel sudut yang dihitung,
//! sisanya FillRect. Hasilnya halus tanpa GDI+ dan tanpa buffer besar yang menetap di RAM.
//! Warna memakai COLORREF 0x00BBGGRR.
#![allow(dead_code)]

use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::win32::*;

pub const fn rgb(r: u32, g: u32, b: u32) -> u32 {
    r | (g << 8) | (b << 16)
}

/// Putih murni, dipakai untuk teks di atas latar merah (tombol Danger).
pub const WHITE: u32 = rgb(0xFF, 0xFF, 0xFF);

/// Warna satu mode tampilan. Nilai mengikuti token Fluent terang dan gelap.
#[derive(Clone, Copy)]
pub struct Pal {
    pub bg: u32,
    pub card: u32,
    pub card_edge: u32,
    pub text: u32,
    pub muted: u32,
    pub subtle: u32,
    pub disabled: u32,
    pub field: u32,
    pub field_edge: u32,
    pub field_line: u32,
    pub field_off: u32,
    pub field_off_edge: u32,
    pub seg_bg: u32,
    pub seg_edge: u32,
    pub seg_hover: u32,
    pub seg_down: u32,
    pub accent: u32,
    pub accent_hover: u32,
    pub accent_down: u32,
    /// Teks dan knob di atas latar aksen. Gelap pada mode malam karena aksen berwarna terang.
    pub on_accent: u32,
    pub danger: u32,
    pub danger_hover: u32,
    pub danger_down: u32,
    pub btn: u32,
    pub btn_hover: u32,
    pub btn_down: u32,
    pub btn_edge: u32,
    pub ghost_hover: u32,
    pub ghost_down: u32,
    pub switch_edge: u32,
    pub knob_off: u32,
    pub off_fill: u32,
    pub focus: u32,
    pub warn: u32,
    pub ok_bg: u32,
    pub ok_fg: u32,
    pub info_bg: u32,
    pub idle_bg: u32,
}

pub static LIGHT: Pal = Pal {
    bg: rgb(0xF3, 0xF3, 0xF3),
    card: rgb(0xFF, 0xFF, 0xFF),
    card_edge: rgb(0xE5, 0xE5, 0xE5),
    text: rgb(0x1A, 0x1A, 0x1A),
    muted: rgb(0x5F, 0x5F, 0x5F),
    subtle: rgb(0x8A, 0x8A, 0x8A),
    disabled: rgb(0xA8, 0xA8, 0xA8),
    field: rgb(0xFF, 0xFF, 0xFF),
    field_edge: rgb(0xD9, 0xD9, 0xD9),
    field_line: rgb(0x8A, 0x8A, 0x8A),
    field_off: rgb(0xF7, 0xF7, 0xF7),
    field_off_edge: rgb(0xE8, 0xE8, 0xE8),
    seg_bg: rgb(0xF3, 0xF3, 0xF3),
    seg_edge: rgb(0xE5, 0xE5, 0xE5),
    seg_hover: rgb(0xE8, 0xE8, 0xE8),
    seg_down: rgb(0xDE, 0xDE, 0xDE),
    accent: rgb(0x00, 0x67, 0xC0),
    accent_hover: rgb(0x19, 0x75, 0xC5),
    accent_down: rgb(0x00, 0x5A, 0x9E),
    on_accent: rgb(0xFF, 0xFF, 0xFF),
    danger: rgb(0xC4, 0x2B, 0x1C),
    danger_hover: rgb(0xD0, 0x3A, 0x2B),
    danger_down: rgb(0xA8, 0x25, 0x18),
    btn: rgb(0xFB, 0xFB, 0xFB),
    btn_hover: rgb(0xF4, 0xF4, 0xF4),
    btn_down: rgb(0xEC, 0xEC, 0xEC),
    btn_edge: rgb(0xD6, 0xD6, 0xD6),
    ghost_hover: rgb(0xE6, 0xE6, 0xE6),
    ghost_down: rgb(0xDC, 0xDC, 0xDC),
    switch_edge: rgb(0x86, 0x86, 0x86),
    knob_off: rgb(0x5C, 0x5C, 0x5C),
    off_fill: rgb(0xC8, 0xC8, 0xC8),
    focus: rgb(0x1A, 0x1A, 0x1A),
    warn: rgb(0xC4, 0x2B, 0x1C),
    ok_bg: rgb(0xDF, 0xF6, 0xDD),
    ok_fg: rgb(0x0F, 0x7B, 0x0F),
    info_bg: rgb(0xDD, 0xEB, 0xF8),
    idle_bg: rgb(0xE6, 0xE6, 0xE6),
};

pub static DARK: Pal = Pal {
    bg: rgb(0x20, 0x20, 0x20),
    card: rgb(0x2B, 0x2B, 0x2B),
    card_edge: rgb(0x33, 0x33, 0x33),
    text: rgb(0xF5, 0xF5, 0xF5),
    muted: rgb(0xB3, 0xB3, 0xB3),
    subtle: rgb(0x8A, 0x8A, 0x8A),
    disabled: rgb(0x5C, 0x5C, 0x5C),
    field: rgb(0x32, 0x32, 0x32),
    field_edge: rgb(0x3A, 0x3A, 0x3A),
    field_line: rgb(0x8E, 0x8E, 0x8E),
    field_off: rgb(0x29, 0x29, 0x29),
    field_off_edge: rgb(0x33, 0x33, 0x33),
    seg_bg: rgb(0x1E, 0x1E, 0x1E),
    seg_edge: rgb(0x2E, 0x2E, 0x2E),
    seg_hover: rgb(0x2C, 0x2C, 0x2C),
    seg_down: rgb(0x38, 0x38, 0x38),
    accent: rgb(0x4C, 0xC2, 0xFF),
    accent_hover: rgb(0x3B, 0xB3, 0xF0),
    accent_down: rgb(0x2F, 0xA3, 0xE0),
    on_accent: rgb(0x0F, 0x17, 0x2A),
    danger: rgb(0xC4, 0x2B, 0x1C),
    danger_hover: rgb(0xD0, 0x3A, 0x2B),
    danger_down: rgb(0xA8, 0x25, 0x18),
    btn: rgb(0x36, 0x36, 0x36),
    btn_hover: rgb(0x3E, 0x3E, 0x3E),
    btn_down: rgb(0x2E, 0x2E, 0x2E),
    btn_edge: rgb(0x4A, 0x4A, 0x4A),
    ghost_hover: rgb(0x33, 0x33, 0x33),
    ghost_down: rgb(0x2A, 0x2A, 0x2A),
    switch_edge: rgb(0x8A, 0x8A, 0x8A),
    knob_off: rgb(0xBA, 0xBA, 0xBA),
    off_fill: rgb(0x4A, 0x4A, 0x4A),
    focus: rgb(0xF5, 0xF5, 0xF5),
    warn: rgb(0xFF, 0x99, 0xA4),
    ok_bg: rgb(0x24, 0x3A, 0x26),
    ok_fg: rgb(0x6C, 0xCB, 0x5F),
    info_bg: rgb(0x26, 0x37, 0x43),
    idle_bg: rgb(0x35, 0x35, 0x35),
};

static DARK_MODE: AtomicBool = AtomicBool::new(false);

/// Mengatur mode gelap untuk seluruh penggambaran. Pemanggil wajib menggambar ulang jendela setelahnya.
pub fn set_dark(on: bool) {
    DARK_MODE.store(on, Ordering::Relaxed);
}

pub fn is_dark() -> bool {
    DARK_MODE.load(Ordering::Relaxed)
}

/// Palet untuk mode yang sedang aktif.
pub fn pal() -> &'static Pal {
    if is_dark() {
        &DARK
    } else {
        &LIGHT
    }
}

/// Glyph Segoe Fluent Icons / Segoe MDL2 Assets.
pub const GLYPH_PLAY: &str = "\u{E768}";
pub const GLYPH_STOP: &str = "\u{E71A}";
pub const GLYPH_RECORD: &str = "\u{E915}";
pub const GLYPH_PICK: &str = "\u{E81D}";
pub const GLYPH_PIN: &str = "\u{E718}";
pub const GLYPH_PINNED: &str = "\u{E840}";
/// Bulan (Segoe MDL2/Fluent "ClearNight") dan matahari ("Sunny") untuk tombol tema.
pub const GLYPH_MOON: &str = "\u{E708}";
pub const GLYPH_SUN: &str = "\u{E706}";

/// Radius sudut dalam piksel logis.
pub const CARD_R: i32 = 12;
pub const CTRL_R: i32 = 7;

#[derive(Clone, Copy, Default)]
pub struct State {
    pub down: bool,
    pub hot: bool,
    pub focus: bool,
    pub off: bool,
}

impl State {
    fn pick(self, rest: u32, hover: u32, down: u32) -> u32 {
        if self.down {
            down
        } else if self.hot {
            hover
        } else {
            rest
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Look {
    Primary,
    Danger,
    Plain,
    Ghost,
}

pub struct Theme {
    pub body: HGDIOBJ,
    pub small: HGDIOBJ,
    pub caption: HGDIOBJ,
    pub strong: HGDIOBJ,
    pub title: HGDIOBJ,
    pub icon: HGDIOBJ,
    pub field: HBRUSH,
    pub field_off: HBRUSH,
}

unsafe fn make_font(face: &str, px: i32, weight: i32) -> HGDIOBJ {
    let f = wide(face);
    CreateFontW(
        -sc(px),
        0,
        0,
        0,
        weight,
        0,
        0,
        0,
        DEFAULT_CHARSET,
        0,
        0,
        CLEARTYPE_QUALITY,
        0,
        f.as_ptr(),
    )
}

/// Font pertama yang benar-benar terpasang; yang terakhir dipakai sebagai cadangan.
unsafe fn first_font(faces: &[&str], px: i32) -> HGDIOBJ {
    let dc = GetDC(null_mut());
    for (i, face) in faces.iter().enumerate() {
        let f = make_font(face, px, FW_NORMAL);
        if i + 1 == faces.len() {
            ReleaseDC(null_mut(), dc);
            return f;
        }
        let old = SelectObject(dc, f);
        let mut buf = [0u16; 64];
        let n = GetTextFaceW(dc, buf.len() as i32, buf.as_mut_ptr()).max(0) as usize;
        SelectObject(dc, old);
        let got = String::from_utf16_lossy(&buf[..n.min(buf.len())]);
        if got.trim_end_matches('\0').eq_ignore_ascii_case(face) {
            ReleaseDC(null_mut(), dc);
            return f;
        }
        DeleteObject(f);
    }
    ReleaseDC(null_mut(), dc);
    null_mut()
}

impl Theme {
    pub fn new() -> Self {
        unsafe {
            let p = pal();
            Theme {
                body: make_font("Segoe UI", 14, FW_NORMAL),
                small: make_font("Segoe UI", 12, FW_NORMAL),
                caption: make_font("Segoe UI", 12, FW_SEMIBOLD),
                strong: make_font("Segoe UI", 14, FW_SEMIBOLD),
                title: make_font("Segoe UI", 20, FW_SEMIBOLD),
                icon: first_font(&["Segoe Fluent Icons", "Segoe MDL2 Assets"], 16),
                field: CreateSolidBrush(p.field),
                field_off: CreateSolidBrush(p.field_off),
            }
        }
    }

    /// Mengganti brush isian setelah mode berubah. Font tidak dibuat ulang karena control anak menyimpannya.
    pub unsafe fn refresh(&mut self) {
        let p = pal();
        DeleteObject(self.field);
        DeleteObject(self.field_off);
        self.field = CreateSolidBrush(p.field);
        self.field_off = CreateSolidBrush(p.field_off);
    }

    /// Tombol membulat dengan glyph opsional. `bg` = warna di belakang tombol.
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn button(
        &self,
        dc: HDC,
        rc: RECT,
        look: Look,
        glyph: &str,
        label: &str,
        st: State,
        bg: u32,
        active: bool,
    ) {
        let p = pal();
        fill(dc, rc, bg);
        let (mut face, mut edge, mut fg) = match look {
            Look::Primary => {
                let c = st.pick(p.accent, p.accent_hover, p.accent_down);
                (c, c, p.on_accent)
            }
            Look::Danger => {
                let c = st.pick(p.danger, p.danger_hover, p.danger_down);
                (c, c, WHITE)
            }
            Look::Plain => (st.pick(p.btn, p.btn_hover, p.btn_down), p.btn_edge, p.text),
            Look::Ghost => {
                let c = st.pick(bg, p.ghost_hover, p.ghost_down);
                (c, c, if active { p.accent } else { p.text })
            }
        };
        if st.off {
            face = p.field_off;
            edge = p.field_off_edge;
            fg = p.disabled;
        }
        let (edge, ew) = if st.focus {
            (p.focus, sc(2))
        } else {
            (edge, 1)
        };
        round_rect(dc, rc, sc(CTRL_R), face, edge, ew, bg);
        let font = if matches!(look, Look::Primary | Look::Danger) {
            self.strong
        } else {
            self.body
        };
        self.glyph_label(dc, rc, glyph, label, font, fg);
    }

    /// Glyph + teks di tengah. Teks dipotong dengan elipsis bila tidak muat.
    unsafe fn glyph_label(
        &self,
        dc: HDC,
        rc: RECT,
        glyph: &str,
        label: &str,
        font: HGDIOBJ,
        fg: u32,
    ) {
        let gw = if glyph.is_empty() {
            0
        } else {
            text_width(dc, glyph, self.icon)
        };
        let gap = if gw > 0 && !label.is_empty() {
            sc(8)
        } else {
            0
        };
        let avail = (rc.right - rc.left - sc(16) - gw - gap).max(0);
        let tw = if label.is_empty() {
            0
        } else {
            text_width(dc, label, font).min(avail)
        };
        let x = rc.left + (rc.right - rc.left - gw - gap - tw) / 2;
        if gw > 0 {
            let g = RECT {
                left: x,
                right: x + gw,
                ..rc
            };
            text(dc, glyph, g, self.icon, fg, DT_LEFT);
        }
        if tw > 0 {
            let t = RECT {
                left: x + gw + gap,
                right: x + gw + gap + tw + 1,
                ..rc
            };
            text(dc, label, t, font, fg, DT_LEFT | DT_END_ELLIPSIS);
        }
    }

    /// Satu segmen pada kontrol segmented. Latar wadahnya digambar oleh jendela induk.
    pub unsafe fn segment(&self, dc: HDC, rc: RECT, label: &str, selected: bool, st: State) {
        let p = pal();
        fill(dc, rc, p.seg_bg);
        let r = sc(CTRL_R - 1);
        let fg = if selected {
            let c = if st.off {
                p.off_fill
            } else {
                st.pick(p.accent, p.accent_hover, p.accent_down)
            };
            let (e, ew) = if st.focus { (p.focus, sc(2)) } else { (c, 1) };
            round_rect(dc, rc, r, c, e, ew, p.seg_bg);
            p.on_accent
        } else {
            if (st.hot || st.down || st.focus) && !st.off {
                let c = st.pick(p.seg_bg, p.seg_hover, p.seg_down);
                let (e, ew) = if st.focus { (p.focus, sc(2)) } else { (c, 1) };
                round_rect(dc, rc, r, c, e, ew, p.seg_bg);
            }
            if st.off {
                p.disabled
            } else {
                p.text
            }
        };
        let font = if selected { self.strong } else { self.body };
        text(dc, label, rc, font, fg, DT_CENTER);
    }

    /// Baris toggle: label (dan petunjuk) di kiri, sakelar di kanan.
    pub unsafe fn toggle(&self, dc: HDC, rc: RECT, label: &str, hint: &str, on: bool, st: State) {
        let p = pal();
        fill(dc, rc, p.card);
        let (tw, th) = (sc(40), sc(20));
        let mid = (rc.top + rc.bottom) / 2;
        let track = RECT {
            left: rc.right - sc(3) - tw,
            top: mid - th / 2,
            right: rc.right - sc(3),
            bottom: mid - th / 2 + th,
        };
        let fg = if st.off { p.disabled } else { p.text };
        let tr = RECT {
            right: track.left - sc(12),
            ..rc
        };
        if hint.is_empty() {
            text(dc, label, tr, self.body, fg, DT_LEFT | DT_END_ELLIPSIS);
        } else {
            text(
                dc,
                label,
                RECT {
                    bottom: mid + sc(1),
                    ..tr
                },
                self.body,
                fg,
                DT_LEFT | DT_END_ELLIPSIS,
            );
            let hc = if st.off { p.disabled } else { p.muted };
            text(
                dc,
                hint,
                RECT {
                    top: mid + sc(1),
                    ..tr
                },
                self.small,
                hc,
                DT_LEFT | DT_END_ELLIPSIS,
            );
        }
        if st.focus {
            let f = inflate(track, sc(3));
            round_rect(
                dc,
                f,
                (f.bottom - f.top) / 2,
                p.card,
                p.focus,
                sc(2),
                p.card,
            );
        }
        let r = th / 2;
        let (face, edge, knob, kx) = if on {
            let c = if st.off {
                p.off_fill
            } else {
                st.pick(p.accent, p.accent_hover, p.accent_down)
            };
            (c, c, p.on_accent, track.right - r)
        } else {
            let face = if st.hot && !st.off {
                p.btn_hover
            } else {
                p.card
            };
            let edge = if st.off { p.disabled } else { p.switch_edge };
            let knob = if st.off { p.disabled } else { p.knob_off };
            (face, edge, knob, track.left + r)
        };
        round_rect(dc, track, r, face, edge, 1, p.card);
        let d = if st.hot && !st.off { sc(14) } else { sc(12) };
        let ky = (track.top + track.bottom) / 2;
        let k = RECT {
            left: kx - d / 2,
            top: ky - d / 2,
            right: kx - d / 2 + d,
            bottom: ky - d / 2 + d,
        };
        round_rect(dc, k, d / 2, knob, knob, 0, face);
    }

    /// Kotak isian di belakang EDIT tanpa border: garis bawah aksen saat fokus, satuan di kanan.
    pub unsafe fn field(
        &self,
        dc: HDC,
        rc: RECT,
        prefix: &str,
        unit: &str,
        focus: bool,
        off: bool,
    ) {
        let p = pal();
        let (face, edge) = if off {
            (p.field_off, p.field_off_edge)
        } else {
            (p.field, p.field_edge)
        };
        let r = sc(CTRL_R - 1);
        round_rect(dc, rc, r, face, edge, 1, p.card);
        if !off {
            let lw = if focus { sc(2) } else { 1 };
            let c = if focus { p.accent } else { p.field_line };
            fill(
                dc,
                RECT {
                    left: rc.left + r,
                    top: rc.bottom - lw,
                    right: rc.right - r,
                    bottom: rc.bottom,
                },
                c,
            );
        }
        let col = if off { p.disabled } else { p.subtle };
        let inner = RECT {
            left: rc.left + sc(8),
            right: rc.right - sc(8),
            ..rc
        };
        if !prefix.is_empty() {
            text(dc, prefix, inner, self.small, col, DT_LEFT);
        }
        if !unit.is_empty() {
            text(dc, unit, inner, self.small, col, DT_RIGHT);
        }
    }

    /// Lencana status: titik berwarna + teks.
    pub unsafe fn pill(&self, dc: HDC, rc: RECT, label: &str, face: u32, fg: u32, bg: u32) {
        let h = rc.bottom - rc.top;
        round_rect(dc, rc, h / 2, face, face, 0, bg);
        let d = sc(8);
        let cx = rc.left + sc(14);
        let cy = (rc.top + rc.bottom) / 2;
        let dot = RECT {
            left: cx - d / 2,
            top: cy - d / 2,
            right: cx - d / 2 + d,
            bottom: cy - d / 2 + d,
        };
        round_rect(dc, dot, d / 2, fg, fg, 0, face);
        let t = RECT {
            left: cx + sc(10),
            right: rc.right - sc(8),
            ..rc
        };
        text(dc, label, t, self.caption, fg, DT_LEFT);
    }
}

impl Drop for Theme {
    fn drop(&mut self) {
        unsafe {
            for f in [
                self.body,
                self.small,
                self.caption,
                self.strong,
                self.title,
                self.icon,
            ] {
                if !f.is_null() {
                    DeleteObject(f);
                }
            }
            DeleteObject(self.field);
            DeleteObject(self.field_off);
        }
    }
}

pub fn inflate(rc: RECT, d: i32) -> RECT {
    RECT {
        left: rc.left - d,
        top: rc.top - d,
        right: rc.right + d,
        bottom: rc.bottom + d,
    }
}

pub unsafe fn fill(dc: HDC, rc: RECT, c: u32) {
    SetDCBrushColor(dc, c);
    FillRect(dc, &rc, GetStockObject(DC_BRUSH));
}

/// Teks satu baris, rata tengah vertikal.
pub unsafe fn text(dc: HDC, s: &str, rc: RECT, font: HGDIOBJ, color: u32, flags: u32) {
    let w: Vec<u16> = s.encode_utf16().collect();
    let mut r = rc;
    let old = SelectObject(dc, font);
    SetBkMode(dc, TRANSPARENT);
    SetTextColor(dc, color);
    DrawTextW(
        dc,
        w.as_ptr(),
        w.len() as i32,
        &mut r,
        flags | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
    );
    SelectObject(dc, old);
}

pub unsafe fn text_width(dc: HDC, s: &str, font: HGDIOBJ) -> i32 {
    let w: Vec<u16> = s.encode_utf16().collect();
    let old = SelectObject(dc, font);
    let mut size = SIZE::default();
    GetTextExtentPoint32W(dc, w.as_ptr(), w.len() as i32, &mut size);
    SelectObject(dc, old);
    size.cx
}

/// Lebar teks tanpa DC yang sedang aktif (dipakai saat menata kontrol).
pub unsafe fn measure(font: HGDIOBJ, s: &str) -> i32 {
    let dc = GetDC(null_mut());
    let w = text_width(dc, s, font);
    ReleaseDC(null_mut(), dc);
    w
}

/// Persegi membulat dengan border `ew` piksel. `bg` dipakai untuk anti-aliasing sudut.
pub unsafe fn round_rect(dc: HDC, rc: RECT, r: i32, face: u32, edge: u32, ew: i32, bg: u32) {
    let w = rc.right - rc.left;
    let h = rc.bottom - rc.top;
    if w <= 0 || h <= 0 {
        return;
    }
    let r = r.min(w / 2).min(h / 2).max(0);
    let ew = ew.max(0);
    let k = r.max(ew);
    fill(
        dc,
        RECT {
            left: rc.left + r,
            right: rc.right - r,
            ..rc
        },
        edge,
    );
    fill(
        dc,
        RECT {
            top: rc.top + r,
            bottom: rc.bottom - r,
            ..rc
        },
        edge,
    );
    fill(
        dc,
        RECT {
            left: rc.left + k,
            top: rc.top + ew,
            right: rc.right - k,
            bottom: rc.bottom - ew,
        },
        face,
    );
    fill(
        dc,
        RECT {
            left: rc.left + ew,
            top: rc.top + k,
            right: rc.right - ew,
            bottom: rc.bottom - k,
        },
        face,
    );
    if r > 0 {
        corners(dc, rc, r, face, edge, ew, bg);
    }
}

/// COLORREF (0x00BBGGRR) ke piksel DIB (0x00RRGGBB).
fn px(c: u32) -> [f32; 3] {
    [
        (c & 0xFF) as f32,
        ((c >> 8) & 0xFF) as f32,
        ((c >> 16) & 0xFF) as f32,
    ]
}

unsafe fn corners(dc: HDC, rc: RECT, r: i32, face: u32, edge: u32, ew: i32, bg: u32) {
    const STACK_R: usize = 32;
    let n = r as usize;
    let area = n * n;
    let mut tl_stack = [0u32; STACK_R * STACK_R];
    let mut buf_stack = [0u32; STACK_R * STACK_R];
    let mut tl_heap;
    let mut buf_heap;
    let (tl, buf): (&mut [u32], &mut [u32]) = if n <= STACK_R {
        (&mut tl_stack[..area], &mut buf_stack[..area])
    } else {
        tl_heap = vec![0u32; area];
        buf_heap = vec![0u32; area];
        (&mut tl_heap, &mut buf_heap)
    };
    let (f, e, b) = (px(face), px(edge), px(bg));
    let rf = r as f32;
    let ri = (r - ew) as f32;
    for j in 0..n {
        for i in 0..n {
            let dx = rf - (i as f32 + 0.5);
            let dy = rf - (j as f32 + 0.5);
            let d = (dx * dx + dy * dy).sqrt();
            let ao = (rf - d + 0.5).clamp(0.0, 1.0);
            let ai = if ew == 0 {
                ao
            } else {
                (ri - d + 0.5).clamp(0.0, 1.0).min(ao)
            };
            let (wb, we, wf) = (1.0 - ao, ao - ai, ai);
            let ch = |k: usize| (b[k] * wb + e[k] * we + f[k] * wf + 0.5) as u32;
            tl[j * n + i] = (ch(0) << 16) | (ch(1) << 8) | ch(2);
        }
    }
    for (fx, fy) in [(false, false), (true, false), (false, true), (true, true)] {
        for j in 0..n {
            for i in 0..n {
                let si = if fx { n - 1 - i } else { i };
                let sj = if fy { n - 1 - j } else { j };
                buf[j * n + i] = tl[sj * n + si];
            }
        }
        let x = if fx { rc.right - r } else { rc.left };
        let y = if fy { rc.bottom - r } else { rc.top };
        blit(dc, x, y, r, r, buf);
    }
}

unsafe fn blit(dc: HDC, x: i32, y: i32, w: i32, h: i32, pixels: &[u32]) {
    let bmi = BITMAPINFO {
        header: BITMAPINFOHEADER {
            size: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            width: w,
            height: -h,
            planes: 1,
            bit_count: 32,
            compression: 0,
            size_image: 0,
            x_ppm: 0,
            y_ppm: 0,
            clr_used: 0,
            clr_important: 0,
        },
        colors: [0],
    };
    StretchDIBits(
        dc,
        x,
        y,
        w,
        h,
        0,
        0,
        w,
        h,
        pixels.as_ptr() as *const c_void,
        &bmi,
        DIB_RGB_COLORS,
        SRCCOPY,
    );
}

/// Gambar di memori lalu salin sekali ke layar (tanpa kedip). Bitmap hanya hidup selama satu paint.
pub struct Buffer {
    target: HDC,
    dc: HDC,
    bmp: HGDIOBJ,
    old: HGDIOBJ,
    rc: RECT,
}

impl Buffer {
    pub unsafe fn new(target: HDC, rc: RECT) -> Buffer {
        let (w, h) = ((rc.right - rc.left).max(1), (rc.bottom - rc.top).max(1));
        let dc = CreateCompatibleDC(target);
        let bmp = if dc.is_null() {
            null_mut()
        } else {
            CreateCompatibleBitmap(target, w, h)
        };
        if bmp.is_null() {
            if !dc.is_null() {
                DeleteDC(dc);
            }
            return Buffer {
                target,
                dc: null_mut(),
                bmp: null_mut(),
                old: null_mut(),
                rc,
            };
        }
        let old = SelectObject(dc, bmp);
        SetViewportOrgEx(dc, -rc.left, -rc.top, null_mut());
        Buffer {
            target,
            dc,
            bmp,
            old,
            rc,
        }
    }

    /// DC untuk menggambar; jatuh ke DC layar bila buffer gagal dibuat.
    pub fn dc(&self) -> HDC {
        if self.dc.is_null() {
            self.target
        } else {
            self.dc
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if self.dc.is_null() {
            return;
        }
        unsafe {
            let (w, h) = (self.rc.right - self.rc.left, self.rc.bottom - self.rc.top);
            BitBlt(
                self.target,
                self.rc.left,
                self.rc.top,
                w,
                h,
                self.dc,
                self.rc.left,
                self.rc.top,
                SRCCOPY,
            );
            SelectObject(self.dc, self.old);
            DeleteObject(self.bmp);
            DeleteDC(self.dc);
        }
    }
}
