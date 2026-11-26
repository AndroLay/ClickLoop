//! Minimal Win32 FFI bindings for ClickLoop-Setup (zero external crates).
#![allow(
    non_snake_case,
    non_camel_case_types,
    dead_code,
    clippy::upper_case_acronyms
)]

use std::ffi::c_void;
use std::path::Path;
use std::ptr::{null, null_mut};

pub type HWND = *mut c_void;
pub type HINSTANCE = *mut c_void;
pub type HMODULE = *mut c_void;
pub type HDC = *mut c_void;
pub type HBRUSH = *mut c_void;
pub type HFONT = *mut c_void;
pub type HICON = *mut c_void;
pub type HCURSOR = *mut c_void;
pub type HMENU = *mut c_void;
pub type HKEY = *mut c_void;
pub type HGDIOBJ = *mut c_void;
pub type HRESULT = i32;
pub type LRESULT = isize;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type BOOL = i32;
pub type DWORD = u32;
pub type UINT = u32;

pub type WNDPROC = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT;

pub const CW_USEDEFAULT: i32 = -2147483648;
pub const SW_SHOW: i32 = 5;
pub const SW_HIDE: i32 = 0;

pub const WS_OVERLAPPED: u32 = 0x0000_0000;
pub const WS_CAPTION: u32 = 0x00C0_0000;
pub const WS_SYSMENU: u32 = 0x0008_0000;
pub const WS_MINIMIZEBOX: u32 = 0x0002_0000;
pub const WS_CHILD: u32 = 0x4000_0000;
pub const WS_VISIBLE: u32 = 0x1000_0000;
pub const WS_TABSTOP: u32 = 0x0001_0000;
pub const WS_CLIPCHILDREN: u32 = 0x0200_0000;
pub const WS_EX_CLIENTEDGE: u32 = 0x0000_0200;

pub const BS_PUSHBUTTON: u32 = 0x0000_0000;
pub const BS_DEFPUSHBUTTON: u32 = 0x0000_0001;
pub const BS_AUTOCHECKBOX: u32 = 0x0000_0003;
pub const ES_READONLY: u32 = 0x0800;
pub const ES_AUTOHSCROLL: u32 = 0x0080;
pub const SS_LEFT: u32 = 0x0000_0000;
pub const SS_NOPREFIX: u32 = 0x0000_0080;

pub const BM_GETCHECK: u32 = 0x00F0;
pub const BM_SETCHECK: u32 = 0x00F1;
pub const BST_CHECKED: usize = 1;
pub const BST_UNCHECKED: usize = 0;

pub const WM_CREATE: u32 = 0x0001;
pub const WM_DESTROY: u32 = 0x0002;
pub const WM_PAINT: u32 = 0x000F;
pub const WM_CLOSE: u32 = 0x0010;
pub const WM_SETFONT: u32 = 0x0030;
pub const WM_KEYDOWN: u32 = 0x0100;
pub const WM_COMMAND: u32 = 0x0111;
pub const WM_CTLCOLOREDIT: u32 = 0x0133;
pub const WM_CTLCOLORBTN: u32 = 0x0135;
pub const WM_CTLCOLORSTATIC: u32 = 0x0138;

pub const VK_ESCAPE: usize = 0x1B;
pub const BN_CLICKED: u16 = 0;

pub const IMAGE_ICON: u32 = 1;
pub const DI_NORMAL: u32 = 0x0003;
pub const IDC_ARROW: usize = 32512;
pub const SM_CXSCREEN: i32 = 0;
pub const SM_CYSCREEN: i32 = 1;

pub const TRANSPARENT: i32 = 1;
pub const OPAQUE: i32 = 2;
pub const DT_LEFT: u32 = 0x0000;
pub const DT_SINGLELINE: u32 = 0x0020;
pub const DT_VCENTER: u32 = 0x0004;
pub const DT_NOPREFIX: u32 = 0x0800;

pub const FW_NORMAL: i32 = 400;
pub const FW_SEMIBOLD: i32 = 600;
pub const DEFAULT_CHARSET: u32 = 1;
pub const OUT_DEFAULT_PRECIS: u32 = 0;
pub const CLIP_DEFAULT_PRECIS: u32 = 0;
pub const CLEARTYPE_QUALITY: u32 = 5;
pub const DEFAULT_PITCH: u32 = 0;

pub const GWLP_USERDATA: i32 = -21;

pub const MB_OK: u32 = 0x0000_0000;
pub const MB_YESNO: u32 = 0x0000_0004;
pub const MB_ICONERROR: u32 = 0x0000_0010;
pub const MB_ICONQUESTION: u32 = 0x0000_0020;
pub const MB_ICONINFORMATION: u32 = 0x0000_0040;
pub const IDYES: i32 = 6;

pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001usize as HKEY;
pub const KEY_WRITE: u32 = 0x0002_0006;
pub const REG_OPTION_NON_VOLATILE: u32 = 0;
pub const REG_SZ: u32 = 1;
pub const REG_DWORD: u32 = 4;
pub const ERROR_SUCCESS: u32 = 0;
pub const ERROR_FILE_NOT_FOUND: u32 = 2;

const COINIT_APARTMENTTHREADED: u32 = 0x2;
const CLSCTX_INPROC_SERVER: u32 = 0x1;
const S_OK: HRESULT = 0;
const S_FALSE: HRESULT = 1;
const RPC_E_CHANGED_MODE: HRESULT = 0x8001_0106u32 as i32;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: UINT,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: DWORD,
    pub pt: POINT,
}

#[repr(C)]
pub struct PAINTSTRUCT {
    pub hdc: HDC,
    pub fErase: BOOL,
    pub rcPaint: RECT,
    pub fRestore: BOOL,
    pub fIncUpdate: BOOL,
    pub rgbReserved: [u8; 32],
}

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cbSize: UINT,
    pub style: UINT,
    pub lpfnWndProc: Option<WNDPROC>,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub hCursor: HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: *const u16,
    pub lpszClassName: *const u16,
    pub hIconSm: HICON,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct GUID {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

const CLSID_SHELL_LINK: GUID = GUID {
    data1: 0x0002_1401,
    data2: 0x0000,
    data3: 0x0000,
    data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

const IID_ISHELL_LINK_W: GUID = GUID {
    data1: 0x0002_14F9,
    data2: 0x0000,
    data3: 0x0000,
    data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

const IID_IPERSIST_FILE: GUID = GUID {
    data1: 0x0000_010B,
    data2: 0x0000,
    data3: 0x0000,
    data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

#[repr(C)]
struct IShellLinkWVtbl {
    query_interface:
        unsafe extern "system" fn(*mut IShellLinkW, *const GUID, *mut *mut c_void) -> HRESULT,
    add_ref: unsafe extern "system" fn(*mut IShellLinkW) -> u32,
    release: unsafe extern "system" fn(*mut IShellLinkW) -> u32,
    get_path: *const c_void,
    get_id_list: *const c_void,
    set_id_list: *const c_void,
    get_description: *const c_void,
    set_description: unsafe extern "system" fn(*mut IShellLinkW, *const u16) -> HRESULT,
    get_working_directory: *const c_void,
    set_working_directory: unsafe extern "system" fn(*mut IShellLinkW, *const u16) -> HRESULT,
    get_arguments: *const c_void,
    set_arguments: *const c_void,
    get_hotkey: *const c_void,
    set_hotkey: *const c_void,
    get_show_cmd: *const c_void,
    set_show_cmd: *const c_void,
    get_icon_location: *const c_void,
    set_icon_location: unsafe extern "system" fn(*mut IShellLinkW, *const u16, i32) -> HRESULT,
    set_relative_path: *const c_void,
    resolve: *const c_void,
    set_path: unsafe extern "system" fn(*mut IShellLinkW, *const u16) -> HRESULT,
}

#[repr(C)]
struct IShellLinkW {
    vtbl: *const IShellLinkWVtbl,
}

#[repr(C)]
struct IPersistFileVtbl {
    query_interface:
        unsafe extern "system" fn(*mut IPersistFile, *const GUID, *mut *mut c_void) -> HRESULT,
    add_ref: unsafe extern "system" fn(*mut IPersistFile) -> u32,
    release: unsafe extern "system" fn(*mut IPersistFile) -> u32,
    get_class_id: *const c_void,
    is_dirty: *const c_void,
    load: *const c_void,
    save: unsafe extern "system" fn(*mut IPersistFile, *const u16, BOOL) -> HRESULT,
    save_completed: *const c_void,
    get_cur_file: *const c_void,
}

#[repr(C)]
struct IPersistFile {
    vtbl: *const IPersistFileVtbl,
}

#[link(name = "user32")]
extern "system" {
    pub fn RegisterClassExW(wc: *const WNDCLASSEXW) -> u16;
    pub fn CreateWindowExW(
        ex_style: DWORD,
        class_name: *const u16,
        window_name: *const u16,
        style: DWORD,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: HWND,
        menu: HMENU,
        instance: HINSTANCE,
        param: *mut c_void,
    ) -> HWND;
    pub fn DefWindowProcW(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> LRESULT;
    pub fn GetMessageW(msg: *mut MSG, hwnd: HWND, min: UINT, max: UINT) -> BOOL;
    pub fn TranslateMessage(msg: *const MSG) -> BOOL;
    pub fn DispatchMessageW(msg: *const MSG) -> LRESULT;
    pub fn IsDialogMessageW(dlg: HWND, msg: *mut MSG) -> BOOL;
    pub fn PostQuitMessage(code: i32);
    pub fn DestroyWindow(hwnd: HWND) -> BOOL;
    pub fn ShowWindow(hwnd: HWND, cmd: i32) -> BOOL;
    pub fn UpdateWindow(hwnd: HWND) -> BOOL;
    pub fn InvalidateRect(hwnd: HWND, rect: *const RECT, erase: BOOL) -> BOOL;
    pub fn SendMessageW(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> LRESULT;
    pub fn PostMessageW(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> BOOL;
    pub fn SetWindowTextW(hwnd: HWND, text: *const u16) -> BOOL;
    pub fn EnableWindow(hwnd: HWND, enable: BOOL) -> BOOL;
    pub fn BeginPaint(hwnd: HWND, ps: *mut PAINTSTRUCT) -> HDC;
    pub fn EndPaint(hwnd: HWND, ps: *const PAINTSTRUCT) -> BOOL;
    pub fn FillRect(hdc: HDC, rect: *const RECT, brush: HBRUSH) -> i32;
    pub fn DrawTextW(hdc: HDC, text: *const u16, len: i32, rect: *mut RECT, format: UINT) -> i32;
    pub fn DrawIconEx(
        hdc: HDC,
        x: i32,
        y: i32,
        icon: HICON,
        cx: i32,
        cy: i32,
        step: UINT,
        flicker_free: HBRUSH,
        flags: UINT,
    ) -> BOOL;
    pub fn LoadImageW(
        hinst: HINSTANCE,
        name: *const u16,
        kind: UINT,
        cx: i32,
        cy: i32,
        flags: UINT,
    ) -> *mut c_void;
    pub fn DestroyIcon(icon: HICON) -> BOOL;
    pub fn LoadCursorW(hinst: HINSTANCE, name: *const u16) -> HCURSOR;
    pub fn GetSystemMetrics(index: i32) -> i32;
    pub fn GetDpiForWindow(hwnd: HWND) -> UINT;
    pub fn AdjustWindowRectExForDpi(
        rect: *mut RECT,
        style: DWORD,
        menu: BOOL,
        ex_style: DWORD,
        dpi: UINT,
    ) -> BOOL;
    pub fn FindWindowW(class_name: *const u16, window_name: *const u16) -> HWND;
    pub fn MessageBoxW(hwnd: HWND, text: *const u16, caption: *const u16, flags: UINT) -> i32;
    pub fn GetWindowLongPtrW(hwnd: HWND, index: i32) -> isize;
    pub fn SetWindowLongPtrW(hwnd: HWND, index: i32, value: isize) -> isize;
}

#[link(name = "gdi32")]
extern "system" {
    pub fn CreateSolidBrush(color: u32) -> HBRUSH;
    pub fn CreateFontW(
        height: i32,
        width: i32,
        escapement: i32,
        orientation: i32,
        weight: i32,
        italic: DWORD,
        underline: DWORD,
        strikeout: DWORD,
        charset: DWORD,
        out_precision: DWORD,
        clip_precision: DWORD,
        quality: DWORD,
        pitch_and_family: DWORD,
        face: *const u16,
    ) -> HFONT;
    pub fn SelectObject(hdc: HDC, obj: HGDIOBJ) -> HGDIOBJ;
    pub fn DeleteObject(obj: HGDIOBJ) -> BOOL;
    pub fn SetBkMode(hdc: HDC, mode: i32) -> i32;
    pub fn SetBkColor(hdc: HDC, color: u32) -> u32;
    pub fn SetTextColor(hdc: HDC, color: u32) -> u32;
}

#[link(name = "kernel32")]
extern "system" {
    pub fn GetModuleHandleW(name: *const u16) -> HMODULE;
    pub fn Sleep(ms: DWORD);
}

#[link(name = "advapi32")]
extern "system" {
    fn RegCreateKeyExW(
        key: HKEY,
        sub_key: *const u16,
        reserved: DWORD,
        class: *const u16,
        options: DWORD,
        sam: DWORD,
        security: *mut c_void,
        result: *mut HKEY,
        disposition: *mut DWORD,
    ) -> u32;
    fn RegSetValueExW(
        key: HKEY,
        value_name: *const u16,
        reserved: DWORD,
        kind: DWORD,
        data: *const u8,
        len: DWORD,
    ) -> u32;
    fn RegDeleteTreeW(key: HKEY, sub_key: *const u16) -> u32;
    fn RegCloseKey(key: HKEY) -> u32;
}

#[link(name = "shell32")]
extern "system" {
    pub fn ShellExecuteW(
        hwnd: HWND,
        operation: *const u16,
        file: *const u16,
        params: *const u16,
        dir: *const u16,
        show: i32,
    ) -> HINSTANCE;
}

#[link(name = "ole32")]
extern "system" {
    fn CoInitializeEx(reserved: *mut c_void, coinit: DWORD) -> HRESULT;
    fn CoUninitialize();
    fn CoCreateInstance(
        rclsid: *const GUID,
        outer: *mut c_void,
        clsctx: DWORD,
        riid: *const GUID,
        out: *mut *mut c_void,
    ) -> HRESULT;
}

pub const fn rgb(r: u32, g: u32, b: u32) -> u32 {
    r | (g << 8) | (b << 16)
}

pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn make_font(px: i32, weight: i32) -> HFONT {
    let face = wide("Segoe UI");
    unsafe {
        CreateFontW(
            -px,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            DEFAULT_PITCH,
            face.as_ptr(),
        )
    }
}

pub fn load_app_icon(size: i32) -> HICON {
    unsafe {
        LoadImageW(
            GetModuleHandleW(null()),
            std::ptr::without_provenance::<u16>(1),
            IMAGE_ICON,
            size,
            size,
            0,
        ) as HICON
    }
}

pub fn message_box(hwnd: HWND, title: &str, text: &str, flags: u32) -> i32 {
    let t = wide(title);
    let m = wide(text);
    unsafe { MessageBoxW(hwnd, m.as_ptr(), t.as_ptr(), flags) }
}

pub fn launch_exe(exe: &Path, work_dir: &Path) -> Result<(), String> {
    let op = wide("open");
    let file = wide(&exe.to_string_lossy());
    let dir = wide(&work_dir.to_string_lossy());
    let res = unsafe {
        ShellExecuteW(
            null_mut(),
            op.as_ptr(),
            file.as_ptr(),
            null(),
            dir.as_ptr(),
            SW_SHOW,
        )
    };
    if (res as isize) <= 32 {
        Err(format!(
            "Failed to launch {} (ShellExecute code {})",
            exe.display(),
            res as isize
        ))
    } else {
        Ok(())
    }
}

/// Creates a Windows `.lnk` shortcut using COM (`IShellLinkW` + `IPersistFile`).
pub fn create_shortcut(
    lnk_path: &Path,
    target: &Path,
    work_dir: &Path,
    description: &str,
) -> Result<(), String> {
    if let Some(parent) = lnk_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create shortcut folder {}: {e}", parent.display()))?;
    }

    unsafe {
        let init_hr = CoInitializeEx(null_mut(), COINIT_APARTMENTTHREADED);
        let need_uninit = init_hr == S_OK || init_hr == S_FALSE;
        if !need_uninit && init_hr != RPC_E_CHANGED_MODE {
            return Err(format!("CoInitializeEx failed (0x{:08X})", init_hr as u32));
        }

        let res = (|| -> Result<(), String> {
            let mut raw_link: *mut c_void = null_mut();
            let hr = CoCreateInstance(
                &CLSID_SHELL_LINK,
                null_mut(),
                CLSCTX_INPROC_SERVER,
                &IID_ISHELL_LINK_W,
                &mut raw_link,
            );
            if hr < 0 || raw_link.is_null() {
                return Err(format!(
                    "CoCreateInstance(ShellLink) failed (0x{:08X})",
                    hr as u32
                ));
            }
            let link = raw_link as *mut IShellLinkW;

            let target_w = wide(&target.to_string_lossy());
            let dir_w = wide(&work_dir.to_string_lossy());
            let desc_w = wide(description);
            let lnk_w = wide(&lnk_path.to_string_lossy());

            let step = (|| -> Result<(), String> {
                let vtbl = &*(*link).vtbl;
                let hr = (vtbl.set_path)(link, target_w.as_ptr());
                if hr < 0 {
                    return Err(format!("IShellLinkW::SetPath failed (0x{:08X})", hr as u32));
                }
                let hr = (vtbl.set_working_directory)(link, dir_w.as_ptr());
                if hr < 0 {
                    return Err(format!(
                        "IShellLinkW::SetWorkingDirectory failed (0x{:08X})",
                        hr as u32
                    ));
                }
                let hr = (vtbl.set_description)(link, desc_w.as_ptr());
                if hr < 0 {
                    return Err(format!(
                        "IShellLinkW::SetDescription failed (0x{:08X})",
                        hr as u32
                    ));
                }
                let hr = (vtbl.set_icon_location)(link, target_w.as_ptr(), 0);
                if hr < 0 {
                    return Err(format!(
                        "IShellLinkW::SetIconLocation failed (0x{:08X})",
                        hr as u32
                    ));
                }

                let mut raw_pf: *mut c_void = null_mut();
                let hr = (vtbl.query_interface)(link, &IID_IPERSIST_FILE, &mut raw_pf);
                if hr < 0 || raw_pf.is_null() {
                    return Err(format!(
                        "QueryInterface(IPersistFile) failed (0x{:08X})",
                        hr as u32
                    ));
                }
                let pf = raw_pf as *mut IPersistFile;
                let pf_vtbl = &*(*pf).vtbl;
                let save_hr = (pf_vtbl.save)(pf, lnk_w.as_ptr(), 1);
                (pf_vtbl.release)(pf);

                if save_hr < 0 {
                    return Err(format!(
                        "IPersistFile::Save failed (0x{:08X})",
                        save_hr as u32
                    ));
                }
                Ok(())
            })();

            ((*(*link).vtbl).release)(link);
            step
        })();

        if need_uninit {
            CoUninitialize();
        }
        res
    }
}

pub fn reg_write_uninstall_entry(
    sub_key: &str,
    strings: &[(&str, &str)],
    dwords: &[(&str, u32)],
) -> Result<(), String> {
    let sub_w = wide(sub_key);
    let mut hkey: HKEY = null_mut();
    let status = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            sub_w.as_ptr(),
            0,
            null(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            null_mut(),
            &mut hkey,
            null_mut(),
        )
    };
    if status != ERROR_SUCCESS {
        return Err(format!("RegCreateKeyExW failed (code {status})"));
    }

    let result = (|| -> Result<(), String> {
        for (name, value) in strings {
            let name_w = wide(name);
            let val_w = wide(value);
            let bytes = (val_w.len() * std::mem::size_of::<u16>()) as DWORD;
            let rc = unsafe {
                RegSetValueExW(
                    hkey,
                    name_w.as_ptr(),
                    0,
                    REG_SZ,
                    val_w.as_ptr() as *const u8,
                    bytes,
                )
            };
            if rc != ERROR_SUCCESS {
                return Err(format!("RegSetValueExW({name}) failed (code {rc})"));
            }
        }
        for (name, value) in dwords {
            let name_w = wide(name);
            let bytes = value.to_le_bytes();
            let rc = unsafe {
                RegSetValueExW(
                    hkey,
                    name_w.as_ptr(),
                    0,
                    REG_DWORD,
                    bytes.as_ptr(),
                    bytes.len() as DWORD,
                )
            };
            if rc != ERROR_SUCCESS {
                return Err(format!("RegSetValueExW({name}) failed (code {rc})"));
            }
        }
        Ok(())
    })();

    unsafe {
        RegCloseKey(hkey);
    }
    result
}

pub fn reg_delete_uninstall_entry(sub_key: &str) -> Result<(), String> {
    let sub_w = wide(sub_key);
    let rc = unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, sub_w.as_ptr()) };
    if rc == ERROR_SUCCESS || rc == ERROR_FILE_NOT_FOUND {
        Ok(())
    } else {
        Err(format!("RegDeleteTreeW failed (code {rc})"))
    }
}
