//! Hotkey: nama yang mudah dibaca, aturan validasi, dan konversi ke format Win32.
use crate::config::Hotkey;
use crate::win32::*;

/// Modifier gaya HOTKEYF (sama dengan nilai kontrol hotkey comctl32).
pub const SHIFT: u32 = 1;
pub const CTRL: u32 = 2;
pub const ALT: u32 = 4;
const EXT: u32 = 8;

/// Modifier untuk RegisterHotKey, selalu dengan MOD_NOREPEAT.
pub fn register_mods(h: Hotkey) -> u32 {
    let mut m = MOD_NOREPEAT;
    if h.mods & SHIFT != 0 {
        m |= MOD_SHIFT;
    }
    if h.mods & CTRL != 0 {
        m |= MOD_CONTROL;
    }
    if h.mods & ALT != 0 {
        m |= MOD_ALT;
    }
    m
}

fn is_extended(vk: u32) -> bool {
    matches!(vk, 0x21..=0x28 | 0x2D | 0x2E | 0x6F | 0x90)
}

/// Tombol yang aman dipakai tanpa Ctrl/Alt karena tidak dipakai mengetik.
fn standalone(vk: u32) -> bool {
    matches!(
        vk,
        0x70..=0x87 | 0x13 | 0x91 | 0x2D | 0x24 | 0x23 | 0x21 | 0x22
    )
}

/// Validasi hotkey global. Huruf, angka, dan simbol tanpa Ctrl/Alt akan mengganggu mengetik.
pub fn check(h: Hotkey) -> Result<(), &'static str> {
    if h.vk == 0 {
        return Err("Press a key for the hotkey.");
    }
    if h.mods & (CTRL | ALT) == 0 && !standalone(h.vk) {
        return Err("Letters, numbers and symbols need Ctrl or Alt.");
    }
    Ok(())
}

/// Contoh: "F6", "Ctrl+Shift+K".
pub fn name(h: Hotkey) -> String {
    let mut s = String::new();
    if h.mods & CTRL != 0 {
        s.push_str("Ctrl+");
    }
    if h.mods & ALT != 0 {
        s.push_str("Alt+");
    }
    if h.mods & SHIFT != 0 {
        s.push_str("Shift+");
    }
    s.push_str(&key_name(h.vk));
    s
}

fn key_name(vk: u32) -> String {
    match vk {
        0x70..=0x87 => format!("F{}", vk - 0x6F),
        0x30..=0x39 | 0x41..=0x5A => char::from(vk as u8).to_string(),
        _ => unsafe {
            let scan = MapVirtualKeyW(vk, 0);
            let mut l = (scan << 16) as i32;
            if is_extended(vk) {
                l |= 1 << 24;
            }
            let mut buf = [0u16; 32];
            let n = GetKeyNameTextW(l, buf.as_mut_ptr(), buf.len() as i32);
            if n > 0 {
                String::from_utf16_lossy(&buf[..n as usize])
            } else {
                format!("Key {vk}")
            }
        },
    }
}

/// Nilai HKM_SETHOTKEY: byte rendah = virtual-key, byte tinggi = modifier.
pub fn to_word(h: Hotkey) -> usize {
    let mut m = h.mods & 7;
    if is_extended(h.vk) {
        m |= EXT;
    }
    (h.vk | (m << 8)) as usize
}

/// Kebalikan dari to_word, dari hasil HKM_GETHOTKEY.
pub fn from_word(w: isize) -> Hotkey {
    Hotkey {
        vk: (w & 0xFF) as u32,
        mods: ((w >> 8) & 7) as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_allows_function_and_modified_keys_but_rejects_plain_typing_keys() {
        assert!(check(Hotkey::plain(0)).is_err());
        assert!(check(Hotkey::plain(b'A' as u32)).is_err());
        assert!(check(Hotkey {
            vk: b'A' as u32,
            mods: SHIFT
        })
        .is_err());
        assert!(check(Hotkey {
            vk: b'A' as u32,
            mods: CTRL
        })
        .is_ok());
        assert!(check(Hotkey {
            vk: b'A' as u32,
            mods: ALT | SHIFT
        })
        .is_ok());
        assert!(check(Hotkey::plain(0x75)).is_ok()); // F6
        assert!(check(Hotkey::plain(0x2D)).is_ok()); // Insert
    }

    #[test]
    fn word_roundtrip_and_register_mods() {
        for h in [
            Hotkey::plain(0x75),
            Hotkey {
                vk: b'K' as u32,
                mods: CTRL | SHIFT,
            },
            Hotkey {
                vk: 0x2D,
                mods: ALT,
            }, // Insert (extended key)
        ] {
            let w = to_word(h);
            assert_eq!(from_word(w as isize), h);
        }
        let m = register_mods(Hotkey {
            vk: 0x75,
            mods: SHIFT | CTRL | ALT,
        });
        assert_eq!(m, MOD_NOREPEAT | MOD_SHIFT | MOD_CONTROL | MOD_ALT);
    }
}
