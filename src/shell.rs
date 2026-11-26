//! Integrasi shell: ikon area notifikasi (tray), menu konteksnya, dan dialog berkas profil.
use std::path::PathBuf;
use std::ptr::null;

use crate::win32::*;

/// Pesan callback dari ikon tray.
pub const WM_TRAY: u32 = WM_APP + 4; // WM_APP+1 dan +2 dipakai picker
const TRAY_ID: u32 = 1;

/// ID item menu tray.
pub const CMD_SHOW: usize = 1;
pub const CMD_TOGGLE: usize = 2;
pub const CMD_EXIT: usize = 3;

unsafe fn tray_data(h: HWND, icon: HICON, tip: &str) -> NotifyIconDataW {
    let mut d: NotifyIconDataW = std::mem::zeroed();
    d.cb_size = std::mem::size_of::<NotifyIconDataW>() as u32;
    d.hwnd = h;
    d.id = TRAY_ID;
    d.flags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    d.callback_message = WM_TRAY;
    d.icon = icon;
    // Sisakan satu unit untuk NUL penutup.
    let t = wide(tip);
    let n = t.len().min(d.tip.len() - 1);
    d.tip[..n].copy_from_slice(&t[..n]);
    d
}

/// Tambah ikon ke area notifikasi.
pub unsafe fn tray_add(h: HWND, icon: HICON, tip: &str) -> bool {
    Shell_NotifyIconW(NIM_ADD, &tray_data(h, icon, tip)) != 0
}

/// Perbarui tooltip ikon tray (mis. status Running / Ready).
pub unsafe fn tray_tip(h: HWND, icon: HICON, tip: &str) {
    Shell_NotifyIconW(NIM_MODIFY, &tray_data(h, icon, tip));
}

/// Hapus ikon dari area notifikasi.
pub unsafe fn tray_remove(h: HWND) {
    let mut d: NotifyIconDataW = std::mem::zeroed();
    d.cb_size = std::mem::size_of::<NotifyIconDataW>() as u32;
    d.hwnd = h;
    d.id = TRAY_ID;
    Shell_NotifyIconW(NIM_DELETE, &d);
}

/// Menu klik kanan di tray. Mengembalikan ID item yang dipilih, atau 0 bila dibatalkan.
pub unsafe fn tray_menu(h: HWND, running: bool) -> usize {
    let menu = CreatePopupMenu();
    if menu.is_null() {
        return 0;
    }
    let show = wide("Show ClickLoop");
    let toggle = wide(if running {
        "Stop clicking"
    } else {
        "Start clicking"
    });
    let exit = wide("Exit");
    AppendMenuW(menu, MF_STRING, CMD_SHOW, show.as_ptr());
    AppendMenuW(menu, MF_STRING, CMD_TOGGLE, toggle.as_ptr());
    AppendMenuW(menu, MF_SEPARATOR, 0, null());
    AppendMenuW(menu, MF_STRING, CMD_EXIT, exit.as_ptr());
    let mut p = POINT { x: 0, y: 0 };
    GetCursorPos(&mut p);
    // Tanpa ini, menu tidak tertutup saat pengguna klik di luar.
    SetForegroundWindow(h);
    let cmd = TrackPopupMenu(
        menu,
        TPM_RIGHTBUTTON | TPM_RETURNCMD,
        p.x,
        p.y,
        0,
        h,
        null(),
    );
    DestroyMenu(menu);
    cmd as usize
}

/// Dialog buka atau simpan profil (.ini). `None` bila dibatalkan.
pub unsafe fn profile_dialog(h: HWND, save: bool) -> Option<PathBuf> {
    file_dialog(
        h,
        save,
        "ClickLoop profile (*.ini)\0*.ini\0All files (*.*)\0*.*\0",
        if save { "Save profile" } else { "Load profile" },
        "ini",
    )
}

/// Dialog buka atau simpan rekaman makro (.clm). `None` bila dibatalkan.
pub unsafe fn macro_dialog(h: HWND, save: bool) -> Option<PathBuf> {
    file_dialog(
        h,
        save,
        "ClickLoop macro (*.clm)\0*.clm\0All files (*.*)\0*.*\0",
        if save { "Save macro" } else { "Load macro" },
        "clm",
    )
}

unsafe fn file_dialog(
    h: HWND,
    save: bool,
    filter_str: &str,
    title_str: &str,
    ext_str: &str,
) -> Option<PathBuf> {
    let filter = wide(filter_str);
    let title = wide(title_str);
    let ext = wide(ext_str);
    let mut buf = [0u16; 520];
    let mut ofn: OpenFileNameW = std::mem::zeroed();
    ofn.struct_size = std::mem::size_of::<OpenFileNameW>() as u32;
    ofn.owner = h;
    ofn.filter = filter.as_ptr();
    ofn.filter_index = 1;
    ofn.file = buf.as_mut_ptr();
    ofn.max_file = buf.len() as u32;
    ofn.title = title.as_ptr();
    ofn.def_ext = ext.as_ptr();
    let common = OFN_EXPLORER | OFN_HIDEREADONLY | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR;
    ofn.flags = if save {
        common | OFN_OVERWRITEPROMPT
    } else {
        common | OFN_FILEMUSTEXIST
    };
    let ok = if save {
        GetSaveFileNameW(&mut ofn)
    } else {
        GetOpenFileNameW(&mut ofn)
    };
    if ok == 0 {
        return None;
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Some(PathBuf::from(String::from_utf16_lossy(&buf[..len])))
}
