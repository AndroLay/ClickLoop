//! Deklarasi Win32 minimal. Tanpa crate eksternal agar binary tetap kecil.
#![allow(non_snake_case, dead_code, clippy::upper_case_acronyms)]

use std::ffi::c_void;

pub type HWND = *mut c_void;
pub type HMENU = *mut c_void;
pub type HINSTANCE = *mut c_void;
pub type HICON = *mut c_void;
pub type HCURSOR = *mut c_void;
pub type HBRUSH = *mut c_void;
pub type HDC = *mut c_void;
pub type HGDIOBJ = *mut c_void;
pub type HANDLE = *mut c_void;
pub type HHOOK = *mut c_void;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type LRESULT = isize;
pub type WndProc = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;
pub type HookProc = unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT;
pub type SubclassProc =
    unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM, usize, usize) -> LRESULT;

// Pesan jendela.
pub const WM_CREATE: u32 = 0x0001;
pub const WM_DESTROY: u32 = 0x0002;
pub const WM_SIZE: u32 = 0x0005;
pub const WM_PAINT: u32 = 0x000F;
pub const WM_CLOSE: u32 = 0x0010;
pub const WM_ERASEBKGND: u32 = 0x0014;
pub const WM_SHOWWINDOW: u32 = 0x0018;
pub const WM_SETFONT: u32 = 0x0030;
pub const WM_DRAWITEM: u32 = 0x002B;
pub const WM_NCDESTROY: u32 = 0x0082;
pub const WM_KEYDOWN: u32 = 0x0100;
pub const WM_KEYUP: u32 = 0x0101;
pub const WM_SYSKEYDOWN: u32 = 0x0104;
pub const WM_SYSKEYUP: u32 = 0x0105;
pub const WM_COMMAND: u32 = 0x0111;
pub const WM_TIMER: u32 = 0x0113;
pub const WM_CHANGEUISTATE: u32 = 0x0127;
pub const WM_CTLCOLOREDIT: u32 = 0x0133;
pub const WM_CTLCOLORSTATIC: u32 = 0x0138;
pub const WM_MOUSEMOVE: u32 = 0x0200;
pub const WM_LBUTTONDOWN: u32 = 0x0201;
pub const WM_LBUTTONUP: u32 = 0x0202;
pub const WM_LBUTTONDBLCLK: u32 = 0x0203;
pub const WM_RBUTTONDOWN: u32 = 0x0204;
pub const WM_RBUTTONUP: u32 = 0x0205;
pub const WM_MBUTTONDOWN: u32 = 0x0207;
pub const WM_MBUTTONUP: u32 = 0x0208;
pub const WM_MOUSEWHEEL: u32 = 0x020A;
pub const WM_MOUSELEAVE: u32 = 0x02A3;
pub const WM_HOTKEY: u32 = 0x0312;
pub const WM_USER: u32 = 0x0400;
pub const WM_APP: u32 = 0x8000;
pub const SIZE_MINIMIZED: usize = 1;
pub const WM_SYSCOMMAND: u32 = 0x0112;
/// Perintah sistem (wParam WM_SYSCOMMAND) dan masknya; bit rendah dipakai sistem.
pub const SC_MINIMIZE: usize = 0xF020;
pub const SC_MASK: usize = 0xFFF0;

// Gaya jendela dan kontrol.
pub const WS_CAPTION: u32 = 0x00C0_0000;
pub const WS_SYSMENU: u32 = 0x0008_0000;
pub const WS_MINIMIZEBOX: u32 = 0x0002_0000;
pub const WS_VISIBLE: u32 = 0x1000_0000;
pub const WS_CHILD: u32 = 0x4000_0000;
pub const WS_TABSTOP: u32 = 0x0001_0000;
pub const WS_CLIPCHILDREN: u32 = 0x0200_0000;
pub const ES_RIGHT: u32 = 0x0000_0002;
pub const ES_AUTOHSCROLL: u32 = 0x0000_0080;
pub const ES_NUMBER: u32 = 0x0000_2000;
pub const EM_LIMITTEXT: u32 = 0x00C5;
pub const EM_SETMARGINS: u32 = 0x00D3;
pub const EC_LEFTMARGIN: usize = 0x0001;
pub const EC_RIGHTMARGIN: usize = 0x0002;
pub const GWL_EXSTYLE: i32 = -20;
pub const WS_EX_CLIENTEDGE: isize = 0x0000_0200;
pub const SWP_NOZORDER: u32 = 0x0004;
pub const SWP_FRAMECHANGED: u32 = 0x0020;
pub const BS_OWNERDRAW: u32 = 0x0000_000B;
pub const CW_USEDEFAULT: i32 = i32::MIN;

// Notifikasi dan ID standar.
pub const BN_CLICKED: u32 = 0;
pub const BN_DOUBLECLICKED: u32 = 5;
pub const EN_CHANGE: u32 = 0x0300;
pub const IDCANCEL: i32 = 2;

// Owner-draw dan GDI.
pub const ODS_SELECTED: u32 = 0x0001;
pub const ODS_DISABLED: u32 = 0x0004;
pub const ODS_FOCUS: u32 = 0x0010;
pub const ODS_NOFOCUSRECT: u32 = 0x0200;
pub const DT_LEFT: u32 = 0x0000;
pub const DT_CENTER: u32 = 0x0001;
pub const DT_RIGHT: u32 = 0x0002;
pub const DT_VCENTER: u32 = 0x0004;
pub const DT_SINGLELINE: u32 = 0x0020;
pub const DT_NOPREFIX: u32 = 0x0800;
pub const DT_END_ELLIPSIS: u32 = 0x8000;
pub const TRANSPARENT: i32 = 1;
pub const FW_NORMAL: i32 = 400;
pub const FW_SEMIBOLD: i32 = 600;
pub const DEFAULT_CHARSET: u32 = 1;
pub const CLEARTYPE_QUALITY: u32 = 5;
pub const DC_BRUSH: i32 = 18;
pub const SRCCOPY: u32 = 0x00CC_0020;
pub const DIB_RGB_COLORS: u32 = 0;
pub const UIS_SET: usize = 1;
pub const UISF_HIDEFOCUS: usize = 1;

// Ikon, posisi jendela, input.
pub const IMAGE_ICON: u32 = 1;
pub const DI_NORMAL: u32 = 3;
pub const SM_CXICON: i32 = 11;
pub const SM_CXSMICON: i32 = 49;
pub const HWND_TOPMOST: isize = -1;
pub const HWND_NOTOPMOST: isize = -2;
pub const SWP_NOSIZE: u32 = 0x0001;
pub const SWP_NOMOVE: u32 = 0x0002;
pub const SWP_NOACTIVATE: u32 = 0x0010;
pub const SW_SHOWNORMAL: i32 = 1;
pub const SW_SHOW: i32 = 5;
pub const SW_RESTORE: i32 = 9;
pub const GWLP_USERDATA: i32 = -21;
pub const DPI_AWARENESS_PER_MONITOR_V2: isize = -4;
pub const IDC_ARROW: usize = 32512;
pub const TME_LEAVE: u32 = 0x0002;
pub const WH_KEYBOARD_LL: i32 = 13;
pub const WH_MOUSE_LL: i32 = 14;
pub const LLMHF_INJECTED: u32 = 0x0001;
pub const LLKHF_EXTENDED: u32 = 0x0001;
pub const LLKHF_INJECTED: u32 = 0x0010;
pub const MONITOR_DEFAULTTONULL: u32 = 0;
pub const VK_SHIFT: i32 = 0x10;
pub const VK_CONTROL: i32 = 0x11;
pub const VK_MENU: i32 = 0x12;
pub const VK_ESCAPE: u32 = 0x1B;
pub const MOD_ALT: u32 = 0x0001;
pub const MOD_CONTROL: u32 = 0x0002;
pub const MOD_SHIFT: u32 = 0x0004;
pub const MOD_NOREPEAT: u32 = 0x4000;
pub const INPUT_MOUSE: u32 = 0;
pub const INPUT_KEYBOARD: u32 = 1;
pub const KEYEVENTF_EXTENDEDKEY: u32 = 0x0001;
pub const KEYEVENTF_KEYUP: u32 = 0x0002;
pub const MOUSEEVENTF_LEFTDOWN: u32 = 0x0002;
pub const MOUSEEVENTF_LEFTUP: u32 = 0x0004;
pub const MOUSEEVENTF_RIGHTDOWN: u32 = 0x0008;
pub const MOUSEEVENTF_RIGHTUP: u32 = 0x0010;
pub const MOUSEEVENTF_MIDDLEDOWN: u32 = 0x0020;
pub const MOUSEEVENTF_MIDDLEUP: u32 = 0x0040;
pub const MOUSEEVENTF_WHEEL: u32 = 0x0800;
pub const MB_ICONERROR: u32 = 0x0000_0010;
pub const ERROR_ALREADY_EXISTS: u32 = 183;
pub const CREATE_WAITABLE_TIMER_HIGH_RESOLUTION: u32 = 0x0000_0002;
pub const TIMER_ALL_ACCESS: u32 = 0x001F_0003;
pub const WAIT_OBJECT_0: u32 = 0;
pub const WAIT_FAILED: u32 = 0xFFFF_FFFF;
pub const INFINITE: u32 = 0xFFFF_FFFF;

// Kontrol hotkey (comctl32).
pub const ICC_HOTKEY_CLASS: u32 = 0x0000_0040;
pub const HKM_SETHOTKEY: u32 = WM_USER + 1;
pub const HKM_GETHOTKEY: u32 = WM_USER + 2;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SIZE {
    pub cx: i32,
    pub cy: i32,
}

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cb_size: u32,
    pub style: u32,
    pub lpfn_wnd_proc: Option<WndProc>,
    pub cb_cls_extra: i32,
    pub cb_wnd_extra: i32,
    pub h_instance: HINSTANCE,
    pub h_icon: HICON,
    pub h_cursor: HCURSOR,
    pub hbr_background: HBRUSH,
    pub lpsz_menu_name: *const u16,
    pub lpsz_class_name: *const u16,
    pub h_icon_sm: HICON,
}

#[repr(C)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: u32,
    pub w_param: WPARAM,
    pub l_param: LPARAM,
    pub time: u32,
    pub pt: POINT,
}

#[repr(C)]
pub struct CreateStructW {
    pub lp_create_params: *mut c_void,
    pub h_instance: HINSTANCE,
    pub h_menu: HMENU,
    pub hwnd_parent: HWND,
    pub cy: i32,
    pub cx: i32,
    pub y: i32,
    pub x: i32,
    pub style: i32,
    pub lpsz_name: *const u16,
    pub lpsz_class: *const u16,
    pub dw_ex_style: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MouseInput {
    pub dx: i32,
    pub dy: i32,
    pub mouse_data: u32,
    pub dw_flags: u32,
    pub time: u32,
    pub dw_extra_info: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeybdInput {
    pub w_vk: u16,
    pub w_scan: u16,
    pub dw_flags: u32,
    pub time: u32,
    pub dw_extra_info: usize,
}

#[repr(C)]
pub union InputUnion {
    pub mi: MouseInput,
    pub ki: KeybdInput,
}

#[repr(C)]
pub struct Input {
    pub ty: u32,
    pub u: InputUnion,
}

#[repr(C)]
pub struct DrawItemStruct {
    pub ctl_type: u32,
    pub ctl_id: u32,
    pub item_id: u32,
    pub item_action: u32,
    pub item_state: u32,
    pub hwnd_item: HWND,
    pub hdc: HDC,
    pub rc_item: RECT,
    pub item_data: usize,
}

#[repr(C)]
pub struct PAINTSTRUCT {
    pub hdc: HDC,
    pub f_erase: i32,
    pub rc_paint: RECT,
    pub f_restore: i32,
    pub f_inc_update: i32,
    pub rgb_reserved: [u8; 32],
}

#[repr(C)]
pub struct BITMAPINFOHEADER {
    pub size: u32,
    pub width: i32,
    pub height: i32,
    pub planes: u16,
    pub bit_count: u16,
    pub compression: u32,
    pub size_image: u32,
    pub x_ppm: i32,
    pub y_ppm: i32,
    pub clr_used: u32,
    pub clr_important: u32,
}

#[repr(C)]
pub struct BITMAPINFO {
    pub header: BITMAPINFOHEADER,
    pub colors: [u32; 1],
}

#[repr(C)]
pub struct TRACKMOUSEEVENT {
    pub cb_size: u32,
    pub flags: u32,
    pub hwnd: HWND,
    pub hover_time: u32,
}

#[repr(C)]
pub struct MSLLHOOKSTRUCT {
    pub pt: POINT,
    pub mouse_data: u32,
    pub flags: u32,
    pub time: u32,
    pub extra: usize,
}

#[repr(C)]
pub struct KBDLLHOOKSTRUCT {
    pub vk_code: u32,
    pub scan_code: u32,
    pub flags: u32,
    pub time: u32,
    pub extra: usize,
}

#[repr(C)]
pub struct INITCOMMONCONTROLSEX {
    pub size: u32,
    pub icc: u32,
}

#[link(name = "user32")]
extern "system" {
    pub fn SetProcessDpiAwarenessContext(value: isize) -> i32;
    pub fn GetDpiForSystem() -> u32;
    pub fn GetDpiForWindow(h: HWND) -> u32;
    pub fn RegisterClassExW(class: *const WNDCLASSEXW) -> u16;
    pub fn CreateWindowExW(
        ex_style: u32,
        class: *const u16,
        name: *const u16,
        style: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        parent: HWND,
        menu: HMENU,
        inst: HINSTANCE,
        param: *mut c_void,
    ) -> HWND;
    pub fn AdjustWindowRect(rect: *mut RECT, style: u32, menu: i32) -> i32;
    pub fn AdjustWindowRectExForDpi(
        rect: *mut RECT,
        style: u32,
        menu: i32,
        ex_style: u32,
        dpi: u32,
    ) -> i32;
    pub fn WindowFromPoint(pt: POINT) -> HWND;
    pub fn RegisterWindowMessageW(name: *const u16) -> u32;
    pub fn DefWindowProcW(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT;
    pub fn ShowWindow(h: HWND, cmd: i32) -> i32;
    pub fn GetMessageW(msg: *mut MSG, h: HWND, min: u32, max: u32) -> i32;
    pub fn TranslateMessage(msg: *const MSG) -> i32;
    pub fn DispatchMessageW(msg: *const MSG) -> LRESULT;
    pub fn IsDialogMessageW(h: HWND, msg: *const MSG) -> i32;
    pub fn PostQuitMessage(code: i32);
    pub fn SendMessageW(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT;
    pub fn PostMessageW(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> i32;
    pub fn GetWindowTextW(h: HWND, buf: *mut u16, max: i32) -> i32;
    pub fn SetWindowTextW(h: HWND, s: *const u16) -> i32;
    pub fn SetWindowLongPtrW(h: HWND, idx: i32, v: isize) -> isize;
    pub fn GetWindowLongPtrW(h: HWND, idx: i32) -> isize;
    pub fn RegisterHotKey(h: HWND, id: i32, mods: u32, vk: u32) -> i32;
    pub fn UnregisterHotKey(h: HWND, id: i32) -> i32;
    pub fn SetTimer(h: HWND, id: usize, ms: u32, f: *const c_void) -> usize;
    pub fn KillTimer(h: HWND, id: usize) -> i32;
    pub fn GetCursorPos(p: *mut POINT) -> i32;
    pub fn SetCursorPos(x: i32, y: i32) -> i32;
    pub fn SendInput(n: u32, inputs: *const Input, size: i32) -> u32;
    pub fn LoadCursorW(inst: HINSTANCE, name: *const u16) -> HCURSOR;
    pub fn LoadImageW(
        inst: HINSTANCE,
        name: *const u16,
        ty: u32,
        cx: i32,
        cy: i32,
        flags: u32,
    ) -> HANDLE;
    pub fn DestroyIcon(icon: HICON) -> i32;
    pub fn DrawIconEx(
        hdc: HDC,
        x: i32,
        y: i32,
        icon: HICON,
        cx: i32,
        cy: i32,
        step: u32,
        brush: HBRUSH,
        flags: u32,
    ) -> i32;
    pub fn MessageBoxW(h: HWND, text: *const u16, caption: *const u16, t: u32) -> i32;
    pub fn FillRect(hdc: HDC, rc: *const RECT, brush: HBRUSH) -> i32;
    pub fn DrawTextW(hdc: HDC, text: *const u16, n: i32, rc: *mut RECT, fmt: u32) -> i32;
    pub fn GetClientRect(h: HWND, rc: *mut RECT) -> i32;
    pub fn GetWindowRect(h: HWND, rc: *mut RECT) -> i32;
    pub fn BeginPaint(h: HWND, ps: *mut PAINTSTRUCT) -> HDC;
    pub fn EndPaint(h: HWND, ps: *const PAINTSTRUCT) -> i32;
    pub fn InvalidateRect(h: HWND, rc: *const RECT, erase: i32) -> i32;
    pub fn GetDC(h: HWND) -> HDC;
    pub fn ReleaseDC(h: HWND, hdc: HDC) -> i32;
    pub fn GetFocus() -> HWND;
    pub fn EnableWindow(h: HWND, enable: i32) -> i32;
    pub fn IsWindowEnabled(h: HWND) -> i32;
    pub fn IsIconic(h: HWND) -> i32;
    pub fn SetWindowPos(h: HWND, after: HWND, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> i32;
    pub fn FindWindowW(class: *const u16, name: *const u16) -> HWND;
    pub fn SetForegroundWindow(h: HWND) -> i32;
    pub fn MonitorFromPoint(pt: POINT, flags: u32) -> HANDLE;
    pub fn GetSystemMetrics(i: i32) -> i32;
    pub fn TrackMouseEvent(t: *mut TRACKMOUSEEVENT) -> i32;
    pub fn SetWindowsHookExW(id: i32, f: HookProc, inst: HINSTANCE, thread: u32) -> HHOOK;
    pub fn UnhookWindowsHookEx(h: HHOOK) -> i32;
    pub fn CallNextHookEx(h: HHOOK, code: i32, w: WPARAM, l: LPARAM) -> LRESULT;
    pub fn GetKeyNameTextW(l: i32, buf: *mut u16, n: i32) -> i32;
    pub fn MapVirtualKeyW(code: u32, map: u32) -> u32;
    pub fn GetAsyncKeyState(vk: i32) -> i16;
    pub fn GetForegroundWindow() -> HWND;
    pub fn GetWindowThreadProcessId(h: HWND, pid: *mut u32) -> u32;
}

#[link(name = "kernel32")]
extern "system" {
    pub fn GetCurrentProcessId() -> u32;
}

#[link(name = "gdi32")]
extern "system" {
    pub fn GetStockObject(i: i32) -> HGDIOBJ;
    pub fn SetDCBrushColor(hdc: HDC, c: u32) -> u32;
    pub fn SelectObject(hdc: HDC, o: HGDIOBJ) -> HGDIOBJ;
    pub fn DeleteObject(o: HGDIOBJ) -> i32;
    pub fn CreateSolidBrush(c: u32) -> HBRUSH;
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC;
    pub fn CreateCompatibleBitmap(hdc: HDC, w: i32, h: i32) -> HGDIOBJ;
    pub fn DeleteDC(hdc: HDC) -> i32;
    pub fn BitBlt(
        dst: HDC,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        src: HDC,
        sx: i32,
        sy: i32,
        rop: u32,
    ) -> i32;
    pub fn SetViewportOrgEx(hdc: HDC, x: i32, y: i32, old: *mut POINT) -> i32;
    pub fn StretchDIBits(
        hdc: HDC,
        xd: i32,
        yd: i32,
        wd: i32,
        hd: i32,
        xs: i32,
        ys: i32,
        ws: i32,
        hs: i32,
        bits: *const c_void,
        bmi: *const BITMAPINFO,
        usage: u32,
        rop: u32,
    ) -> i32;
    pub fn SetTextColor(hdc: HDC, c: u32) -> u32;
    pub fn SetBkColor(hdc: HDC, c: u32) -> u32;
    pub fn SetBkMode(hdc: HDC, mode: i32) -> i32;
    pub fn GetTextExtentPoint32W(hdc: HDC, s: *const u16, n: i32, size: *mut SIZE) -> i32;
    pub fn GetTextFaceW(hdc: HDC, n: i32, buf: *mut u16) -> i32;
    pub fn CreateFontW(
        h: i32,
        w: i32,
        esc: i32,
        ori: i32,
        weight: i32,
        italic: u32,
        under: u32,
        strike: u32,
        charset: u32,
        out_p: u32,
        clip_p: u32,
        quality: u32,
        pitch: u32,
        face: *const u16,
    ) -> HGDIOBJ;
}

#[link(name = "comctl32")]
extern "system" {
    pub fn InitCommonControlsEx(icc: *const INITCOMMONCONTROLSEX) -> i32;
    pub fn SetWindowSubclass(h: HWND, f: SubclassProc, id: usize, data: usize) -> i32;
    pub fn RemoveWindowSubclass(h: HWND, f: SubclassProc, id: usize) -> i32;
    pub fn DefSubclassProc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT;
}

#[link(name = "kernel32")]
extern "system" {
    pub fn GetModuleHandleW(name: *const u16) -> HINSTANCE;
    pub fn GetCurrentProcess() -> HANDLE;
    pub fn K32EmptyWorkingSet(h: HANDLE) -> i32;
    pub fn CreateMutexW(attr: *mut c_void, owner: i32, name: *const u16) -> HANDLE;
    pub fn GetLastError() -> u32;
    pub fn CreateEventW(attr: *mut c_void, manual: i32, initial: i32, name: *const u16) -> HANDLE;
    pub fn SetEvent(h: HANDLE) -> i32;
    pub fn CloseHandle(h: HANDLE) -> i32;
    pub fn CreateWaitableTimerExW(
        attr: *mut c_void,
        name: *const u16,
        flags: u32,
        access: u32,
    ) -> HANDLE;
    pub fn SetWaitableTimer(
        h: HANDLE,
        due: *const i64,
        period: i32,
        routine: *const c_void,
        arg: *mut c_void,
        resume: i32,
    ) -> i32;
    pub fn WaitForSingleObject(h: HANDLE, ms: u32) -> u32;
    pub fn WaitForMultipleObjects(n: u32, hs: *const HANDLE, all: i32, ms: u32) -> u32;
}

#[link(name = "winmm")]
extern "system" {
    pub fn timeBeginPeriod(p: u32) -> u32;
    pub fn timeEndPeriod(p: u32) -> u32;
}

// ---- Tray, menu popup, dan dialog berkas ----

pub const NIM_ADD: u32 = 0;
pub const NIM_MODIFY: u32 = 1;
pub const NIM_DELETE: u32 = 2;
pub const NIF_MESSAGE: u32 = 0x1;
pub const NIF_ICON: u32 = 0x2;
pub const NIF_TIP: u32 = 0x4;
pub const MF_STRING: u32 = 0x0000;
pub const MF_SEPARATOR: u32 = 0x0800;
pub const TPM_RIGHTBUTTON: u32 = 0x0002;
pub const TPM_RETURNCMD: u32 = 0x0100;
pub const OFN_HIDEREADONLY: u32 = 0x0004;
pub const OFN_NOCHANGEDIR: u32 = 0x0008;
pub const OFN_OVERWRITEPROMPT: u32 = 0x0002;
pub const OFN_PATHMUSTEXIST: u32 = 0x0800;
pub const OFN_FILEMUSTEXIST: u32 = 0x1000;
pub const OFN_EXPLORER: u32 = 0x0008_0000;

/// NOTIFYICONDATAW dengan layout x64 (976 byte, termasuk hBalloonIcon).
#[allow(dead_code)]
#[repr(C)]
pub struct NotifyIconDataW {
    pub cb_size: u32,
    pub hwnd: HWND,
    pub id: u32,
    pub flags: u32,
    pub callback_message: u32,
    pub icon: HICON,
    pub tip: [u16; 128],
    pub state: u32,
    pub state_mask: u32,
    pub info: [u16; 256],
    pub timeout_or_version: u32,
    pub info_title: [u16; 64],
    pub info_flags: u32,
    pub guid_item: [u8; 16],
    pub balloon_icon: HICON,
}

/// OPENFILENAMEW dengan layout x64 (152 byte).
#[allow(dead_code)]
#[repr(C)]
pub struct OpenFileNameW {
    pub struct_size: u32,
    pub owner: HWND,
    pub instance: HINSTANCE,
    pub filter: *const u16,
    pub custom_filter: *mut u16,
    pub max_custom_filter: u32,
    pub filter_index: u32,
    pub file: *mut u16,
    pub max_file: u32,
    pub file_title: *mut u16,
    pub max_file_title: u32,
    pub initial_dir: *const u16,
    pub title: *const u16,
    pub flags: u32,
    pub file_offset: u16,
    pub file_extension: u16,
    pub def_ext: *const u16,
    pub cust_data: isize,
    pub hook: *const c_void,
    pub template_name: *const u16,
    pub reserved: *mut c_void,
    pub reserved2: u32,
    pub flags_ex: u32,
}

#[link(name = "shell32")]
extern "system" {
    pub fn Shell_NotifyIconW(msg: u32, data: *const NotifyIconDataW) -> i32;
}

#[link(name = "comdlg32")]
extern "system" {
    pub fn GetOpenFileNameW(ofn: *mut OpenFileNameW) -> i32;
    pub fn GetSaveFileNameW(ofn: *mut OpenFileNameW) -> i32;
}

#[link(name = "user32")]
extern "system" {
    pub fn CreatePopupMenu() -> HMENU;
    pub fn AppendMenuW(menu: HMENU, flags: u32, id: usize, text: *const u16) -> i32;
    pub fn TrackPopupMenu(
        menu: HMENU,
        flags: u32,
        x: i32,
        y: i32,
        reserved: i32,
        h: HWND,
        rc: *const c_void,
    ) -> i32;
    pub fn DestroyMenu(menu: HMENU) -> i32;
    pub fn DestroyWindow(h: HWND) -> i32;
    pub fn MoveWindow(h: HWND, x: i32, y: i32, w: i32, height: i32, repaint: i32) -> i32;
    pub fn SetFocus(h: HWND) -> HWND;
    pub fn IsWindowVisible(h: HWND) -> i32;
    pub fn GetWindow(h: HWND, cmd: u32) -> HWND;
}

#[link(name = "dwmapi")]
extern "system" {
    pub fn DwmSetWindowAttribute(h: HWND, attr: u32, value: *const c_void, size: u32) -> i32;
}

/// Atribut DWM untuk judul jendela gelap (Windows 10 build 1809 ke atas dan Windows 11).
pub const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;

pub const SW_HIDE: i32 = 0;
pub const GW_CHILD: u32 = 5;
pub const GW_HWNDNEXT: u32 = 2;

/// String UTF-16 berakhir null untuk API Win32.
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Menyesuaikan persegi klien ke ukuran jendela utuh sesuai DPI aktif.
pub unsafe fn adjust_rect(rc: &mut RECT, style: u32, d: u32) {
    if AdjustWindowRectExForDpi(rc, style, 0, 0, d) == 0 {
        AdjustWindowRect(rc, style, 0);
    }
}

pub const WM_DPICHANGED: u32 = 0x02E0;

/// DPI aktif jendela utama (monitor tempat jendela berada). 0 berarti belum ditetapkan.
static DPI: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Menetapkan DPI aktif. Dipanggil saat jendela dibuat dan saat pindah ke monitor dengan skala lain.
pub fn set_dpi(d: u32) {
    if d != 0 {
        DPI.store(d, std::sync::atomic::Ordering::Relaxed);
    }
}

/// DPI aktif; jika belum ditetapkan, memakai DPI sistem.
pub fn dpi() -> u32 {
    match DPI.load(std::sync::atomic::Ordering::Relaxed) {
        0 => unsafe { GetDpiForSystem() },
        d => d,
    }
}

/// Skala piksel logis (96 DPI) ke DPI aktif.
pub fn sc(v: i32) -> i32 {
    v * dpi() as i32 / 96
}

pub unsafe fn text_of(h: HWND) -> String {
    let mut buf = [0u16; 256];
    let n = GetWindowTextW(h, buf.as_mut_ptr(), buf.len() as i32);
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

pub unsafe fn set_text(h: HWND, s: &str) {
    let w = wide(s);
    SetWindowTextW(h, w.as_ptr());
}

pub unsafe fn show_error(h: HWND, text: &str) {
    let t = wide(text);
    let c = wide("ClickLoop");
    MessageBoxW(h, t.as_ptr(), c.as_ptr(), MB_ICONERROR);
}
