//! ClickLoop-Setup: standalone Windows installer and uninstaller in pure Rust.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod install;
mod win32;

use std::ptr::{null, null_mut};

use install::{CliAction, InstallOptions, APP_NAME, APP_VERSION};
use win32::*;

const CLASS_NAME: &str = "ClickLoopSetupWindow";

const BASE_W: i32 = 468;
const BASE_H: i32 = 342;
const HEADER_H: i32 = 76;
const FOOTER_Y: i32 = 282;

const COL_HEADER: u32 = rgb(0xFF, 0xFF, 0xFF);
const COL_BG: u32 = rgb(0xF3, 0xF3, 0xF3);
const COL_FOOTER: u32 = rgb(0xFF, 0xFF, 0xFF);
const COL_DIVIDER: u32 = rgb(0xE1, 0xE1, 0xE1);
const COL_TEXT: u32 = rgb(0x1B, 0x1B, 0x1B);
const COL_MUTED: u32 = rgb(0x5F, 0x5F, 0x5F);
const COL_ACCENT: u32 = rgb(0x00, 0x5F, 0xB8);

const ID_CHK_START_MENU: u16 = 101;
const ID_CHK_DESKTOP: u16 = 102;
const ID_CHK_LAUNCH: u16 = 103;
const ID_CHK_REMOVE_CFG: u16 = 104;
const ID_BTN_PRIMARY: u16 = 201;
const ID_BTN_SECONDARY: u16 = 202;
const ID_BTN_UNINSTALL: u16 = 203;

struct SetupUi {
    hwnd: HWND,
    dpi: i32,
    uninstall_mode: bool,
    installed: bool,
    finished: bool,
    font_title: HFONT,
    font_body: HFONT,
    font_small: HFONT,
    brush_bg: HBRUSH,
    brush_header: HBRUSH,
    brush_footer: HBRUSH,
    brush_divider: HBRUSH,
    icon_header: HICON,
    lbl_status: HWND,
    chk_start_menu: HWND,
    chk_desktop: HWND,
    chk_launch: HWND,
    chk_remove_cfg: HWND,
    btn_primary: HWND,
    btn_secondary: HWND,
    btn_uninstall: HWND,
}

impl SetupUi {
    fn s(&self, px: i32) -> i32 {
        (px * self.dpi + 48) / 96
    }

    fn set_text(&self, h: HWND, text: &str) {
        if !h.is_null() {
            let w = wide(text);
            unsafe {
                SetWindowTextW(h, w.as_ptr());
            }
        }
    }

    fn checked(&self, h: HWND) -> bool {
        !h.is_null() && unsafe { SendMessageW(h, BM_GETCHECK, 0, 0) as usize == BST_CHECKED }
    }

    fn set_checked(&self, h: HWND, checked: bool) {
        if !h.is_null() {
            let state = if checked { BST_CHECKED } else { BST_UNCHECKED };
            unsafe {
                SendMessageW(h, BM_SETCHECK, state, 0);
            }
        }
    }

    fn set_enabled(&self, h: HWND, enabled: bool) {
        if !h.is_null() {
            unsafe {
                EnableWindow(h, if enabled { 1 } else { 0 });
            }
        }
    }

    fn create_child(
        &self,
        class: &str,
        text: &str,
        styles: (u32, u32),
        bounds: [i32; 4],
        id: u16,
        font: HFONT,
    ) -> HWND {
        let (style, ex_style) = styles;
        let [x, y, w, h] = bounds;
        let cls = wide(class);
        let txt = wide(text);
        let ctrl = unsafe {
            CreateWindowExW(
                ex_style,
                cls.as_ptr(),
                txt.as_ptr(),
                WS_CHILD | WS_VISIBLE | style,
                self.s(x),
                self.s(y),
                self.s(w),
                self.s(h),
                self.hwnd,
                id as usize as HMENU,
                GetModuleHandleW(null()),
                null_mut(),
            )
        };
        if !ctrl.is_null() && !font.is_null() {
            unsafe {
                SendMessageW(ctrl, WM_SETFONT, font as WPARAM, 1);
            }
        }
        ctrl
    }

    fn build_controls(&mut self) {
        let dir_text = install::install_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|e| e);

        self.create_child(
            "STATIC",
            "Install folder",
            (SS_LEFT | SS_NOPREFIX, 0),
            [24, 92, 420, 18],
            0,
            self.font_small,
        );

        self.create_child(
            "EDIT",
            &dir_text,
            (ES_READONLY | ES_AUTOHSCROLL | WS_TABSTOP, WS_EX_CLIENTEDGE),
            [24, 112, 420, 24],
            0,
            self.font_body,
        );

        let initial_status = if self.uninstall_mode {
            "This will remove ClickLoop, its shortcuts, and its Windows Apps entry."
        } else if self.installed {
            "ClickLoop is already installed. Click Update to install the latest version."
        } else {
            "Installs for your Windows account — no administrator rights required."
        };

        self.lbl_status = self.create_child(
            "STATIC",
            initial_status,
            (SS_LEFT | SS_NOPREFIX, 0),
            [24, 146, 420, 34],
            0,
            self.font_small,
        );

        if self.uninstall_mode {
            self.chk_remove_cfg = self.create_child(
                "BUTTON",
                "Also delete saved settings (clickloop.ini)",
                (BS_AUTOCHECKBOX | WS_TABSTOP, 0),
                [24, 192, 420, 24],
                ID_CHK_REMOVE_CFG,
                self.font_body,
            );
            self.set_checked(self.chk_remove_cfg, false);
        } else {
            self.chk_start_menu = self.create_child(
                "BUTTON",
                "Create Start Menu shortcut",
                (BS_AUTOCHECKBOX | WS_TABSTOP, 0),
                [24, 186, 420, 22],
                ID_CHK_START_MENU,
                self.font_body,
            );
            self.chk_desktop = self.create_child(
                "BUTTON",
                "Create Desktop shortcut",
                (BS_AUTOCHECKBOX | WS_TABSTOP, 0),
                [24, 214, 420, 22],
                ID_CHK_DESKTOP,
                self.font_body,
            );
            self.chk_launch = self.create_child(
                "BUTTON",
                "Launch ClickLoop after installation",
                (BS_AUTOCHECKBOX | WS_TABSTOP, 0),
                [24, 242, 420, 22],
                ID_CHK_LAUNCH,
                self.font_body,
            );
            self.set_checked(self.chk_start_menu, true);
            self.set_checked(self.chk_desktop, true);
            self.set_checked(self.chk_launch, true);
        }

        if !self.uninstall_mode {
            self.btn_uninstall = self.create_child(
                "BUTTON",
                "Uninstall",
                (BS_PUSHBUTTON | WS_TABSTOP, 0),
                [24, 298, 92, 28],
                ID_BTN_UNINSTALL,
                self.font_body,
            );
            self.set_enabled(self.btn_uninstall, self.installed);
        }

        self.btn_secondary = self.create_child(
            "BUTTON",
            "Cancel",
            (BS_PUSHBUTTON | WS_TABSTOP, 0),
            [258, 298, 88, 28],
            ID_BTN_SECONDARY,
            self.font_body,
        );

        let primary_label = if self.uninstall_mode {
            "Uninstall"
        } else if self.installed {
            "Update"
        } else {
            "Install"
        };
        self.btn_primary = self.create_child(
            "BUTTON",
            primary_label,
            (BS_DEFPUSHBUTTON | WS_TABSTOP, 0),
            [354, 298, 90, 28],
            ID_BTN_PRIMARY,
            self.font_body,
        );
    }

    fn handle_primary(&mut self) {
        if self.finished {
            unsafe {
                DestroyWindow(self.hwnd);
            }
            return;
        }

        if self.uninstall_mode {
            let remove_cfg = self.checked(self.chk_remove_cfg);
            self.run_uninstall(remove_cfg);
            return;
        }

        let opts = InstallOptions {
            start_menu: self.checked(self.chk_start_menu),
            desktop: self.checked(self.chk_desktop),
            launch: self.checked(self.chk_launch),
        };

        self.set_enabled(self.btn_primary, false);
        self.set_enabled(self.btn_uninstall, false);
        self.set_text(self.lbl_status, "Installing ClickLoop...");
        unsafe {
            UpdateWindow(self.hwnd);
        }

        match install::install(opts) {
            Ok(report) => {
                self.installed = true;
                self.finished = true;
                let verb = if report.updated {
                    "updated"
                } else {
                    "installed"
                };
                self.set_text(
                    self.lbl_status,
                    &format!(
                        "ClickLoop {APP_VERSION} was {verb} successfully in {}.",
                        report.install_dir.display()
                    ),
                );
                self.set_enabled(self.chk_start_menu, false);
                self.set_enabled(self.chk_desktop, false);
                self.set_enabled(self.chk_launch, false);
                self.set_enabled(self.btn_secondary, false);
                self.set_text(self.btn_primary, "Finish");
                self.set_enabled(self.btn_primary, true);
            }
            Err(err) => {
                self.set_text(self.lbl_status, &format!("Installation failed: {err}"));
                self.set_enabled(self.btn_primary, true);
                self.set_enabled(self.btn_uninstall, self.installed);
                message_box(self.hwnd, "ClickLoop Setup", &err, MB_OK | MB_ICONERROR);
            }
        }
    }

    fn run_uninstall(&mut self, remove_cfg: bool) {
        self.set_enabled(self.btn_primary, false);
        self.set_enabled(self.btn_uninstall, false);
        self.set_text(self.lbl_status, "Removing ClickLoop...");
        unsafe {
            UpdateWindow(self.hwnd);
        }

        match install::uninstall(remove_cfg) {
            Ok(_report) => {
                self.installed = false;
                self.finished = true;
                self.set_text(
                    self.lbl_status,
                    "ClickLoop has been uninstalled cleanly from your system.",
                );
                self.set_enabled(self.chk_start_menu, false);
                self.set_enabled(self.chk_desktop, false);
                self.set_enabled(self.chk_launch, false);
                self.set_enabled(self.chk_remove_cfg, false);
                self.set_enabled(self.btn_secondary, false);
                self.set_text(self.btn_primary, "Finish");
                self.set_enabled(self.btn_primary, true);
            }
            Err(err) => {
                self.set_text(self.lbl_status, &format!("Uninstall failed: {err}"));
                self.set_enabled(self.btn_primary, true);
                self.set_enabled(self.btn_uninstall, self.installed);
                message_box(self.hwnd, "ClickLoop Setup", &err, MB_OK | MB_ICONERROR);
            }
        }
    }

    fn handle_uninstall_button(&mut self) {
        let confirm = message_box(
            self.hwnd,
            "Uninstall ClickLoop",
            "Remove ClickLoop, its Start Menu/Desktop shortcuts, and its Apps entry?\n\nYour clickloop.ini settings file will be kept.",
            MB_YESNO | MB_ICONQUESTION,
        );
        if confirm == IDYES {
            self.run_uninstall(false);
        }
    }

    fn paint(&self) {
        let mut ps: PAINTSTRUCT = unsafe { std::mem::zeroed() };
        let hdc = unsafe { BeginPaint(self.hwnd, &mut ps) };

        let w = self.s(BASE_W);
        let h = self.s(BASE_H);
        let hh = self.s(HEADER_H);
        let fy = self.s(FOOTER_Y);

        let r_header = RECT {
            left: 0,
            top: 0,
            right: w,
            bottom: hh,
        };
        let r_div1 = RECT {
            left: 0,
            top: hh,
            right: w,
            bottom: hh + 1,
        };
        let r_body = RECT {
            left: 0,
            top: hh + 1,
            right: w,
            bottom: fy,
        };
        let r_div2 = RECT {
            left: 0,
            top: fy,
            right: w,
            bottom: fy + 1,
        };
        let r_footer = RECT {
            left: 0,
            top: fy + 1,
            right: w,
            bottom: h,
        };

        unsafe {
            FillRect(hdc, &r_header, self.brush_header);
            FillRect(hdc, &r_div1, self.brush_divider);
            FillRect(hdc, &r_body, self.brush_bg);
            FillRect(hdc, &r_div2, self.brush_divider);
            FillRect(hdc, &r_footer, self.brush_footer);

            if !self.icon_header.is_null() {
                let ico_sz = self.s(40);
                DrawIconEx(
                    hdc,
                    self.s(24),
                    self.s(18),
                    self.icon_header,
                    ico_sz,
                    ico_sz,
                    0,
                    null_mut(),
                    DI_NORMAL,
                );
            }

            SetBkMode(hdc, TRANSPARENT);

            let title = if self.uninstall_mode {
                format!("Uninstall {APP_NAME}")
            } else {
                format!("{APP_NAME} Setup")
            };
            let title_w = wide(&title);
            let old_font = SelectObject(hdc, self.font_title);
            SetTextColor(hdc, COL_TEXT);
            let mut r_title = RECT {
                left: self.s(76),
                top: self.s(16),
                right: w - self.s(20),
                bottom: self.s(42),
            };
            DrawTextW(
                hdc,
                title_w.as_ptr(),
                -1,
                &mut r_title,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
            );

            let sub = format!("Version {APP_VERSION}  •  Auto Clicker & Macro Recorder");
            let sub_w = wide(&sub);
            SelectObject(hdc, self.font_small);
            SetTextColor(hdc, COL_MUTED);
            let mut r_sub = RECT {
                left: self.s(76),
                top: self.s(42),
                right: w - self.s(20),
                bottom: self.s(62),
            };
            DrawTextW(
                hdc,
                sub_w.as_ptr(),
                -1,
                &mut r_sub,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
            );

            SelectObject(hdc, old_font);
            EndPaint(self.hwnd, &ps);
        }
    }
}

impl Drop for SetupUi {
    fn drop(&mut self) {
        unsafe {
            if !self.font_title.is_null() {
                DeleteObject(self.font_title);
            }
            if !self.font_body.is_null() {
                DeleteObject(self.font_body);
            }
            if !self.font_small.is_null() {
                DeleteObject(self.font_small);
            }
            if !self.brush_bg.is_null() {
                DeleteObject(self.brush_bg);
            }
            if !self.brush_header.is_null() {
                DeleteObject(self.brush_header);
            }
            if !self.brush_footer.is_null() {
                DeleteObject(self.brush_footer);
            }
            if !self.brush_divider.is_null() {
                DeleteObject(self.brush_divider);
            }
            if !self.icon_header.is_null() {
                DestroyIcon(self.icon_header);
            }
        }
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> LRESULT {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SetupUi;

    match msg {
        WM_PAINT if !ptr.is_null() => {
            (*ptr).paint();
            0
        }
        WM_CTLCOLORSTATIC | WM_CTLCOLORBTN if !ptr.is_null() => {
            let ui = &*ptr;
            let hdc = w as HDC;
            let ctrl = l as HWND;
            if ctrl == ui.btn_primary || ctrl == ui.btn_secondary || ctrl == ui.btn_uninstall {
                SetBkMode(hdc, OPAQUE);
                SetBkColor(hdc, COL_FOOTER);
                return ui.brush_footer as LRESULT;
            }
            if ctrl == ui.lbl_status {
                SetTextColor(hdc, if ui.finished { COL_ACCENT } else { COL_MUTED });
            } else {
                SetTextColor(hdc, COL_TEXT);
            }
            SetBkMode(hdc, OPAQUE);
            SetBkColor(hdc, COL_BG);
            ui.brush_bg as LRESULT
        }
        WM_COMMAND if !ptr.is_null() => {
            let id = (w & 0xFFFF) as u16;
            let code = ((w >> 16) & 0xFFFF) as u16;
            if code == BN_CLICKED {
                match id {
                    ID_BTN_PRIMARY => (*ptr).handle_primary(),
                    ID_BTN_SECONDARY => {
                        DestroyWindow(hwnd);
                    }
                    ID_BTN_UNINSTALL => (*ptr).handle_uninstall_button(),
                    _ => {}
                }
            }
            0
        }
        WM_KEYDOWN if w == VK_ESCAPE => {
            DestroyWindow(hwnd);
            0
        }
        WM_CLOSE => {
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            if !ptr.is_null() {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                drop(Box::from_raw(ptr));
            }
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, w, l),
    }
}

fn run_gui(uninstall_mode: bool) {
    unsafe {
        let hinst = GetModuleHandleW(null());
        let cls = wide(CLASS_NAME);
        let icon_big = load_app_icon(32);
        let icon_sm = load_app_icon(16);

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as UINT,
            style: 0,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinst,
            hIcon: icon_big,
            hCursor: LoadCursorW(null_mut(), std::ptr::without_provenance::<u16>(IDC_ARROW)),
            hbrBackground: null_mut(),
            lpszMenuName: null(),
            lpszClassName: cls.as_ptr(),
            hIconSm: icon_sm,
        };
        if RegisterClassExW(&wc) == 0 {
            message_box(
                null_mut(),
                "ClickLoop Setup",
                "Failed to register setup window.",
                MB_OK | MB_ICONERROR,
            );
            return;
        }

        let title = if uninstall_mode {
            format!("Uninstall {APP_NAME}")
        } else {
            format!("{APP_NAME} {APP_VERSION} Setup")
        };
        let title_w = wide(&title);
        let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_CLIPCHILDREN;

        let hwnd = CreateWindowExW(
            0,
            cls.as_ptr(),
            title_w.as_ptr(),
            style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            BASE_W,
            BASE_H,
            null_mut(),
            null_mut(),
            hinst,
            null_mut(),
        );
        if hwnd.is_null() {
            message_box(
                null_mut(),
                "ClickLoop Setup",
                "Failed to create setup window.",
                MB_OK | MB_ICONERROR,
            );
            return;
        }

        let dpi = (GetDpiForWindow(hwnd) as i32).max(96);
        let mut rc = RECT {
            left: 0,
            top: 0,
            right: (BASE_W * dpi + 48) / 96,
            bottom: (BASE_H * dpi + 48) / 96,
        };
        AdjustWindowRectExForDpi(&mut rc, style, 0, 0, dpi as UINT);
        let win_w = rc.right - rc.left;
        let win_h = rc.bottom - rc.top;
        let sx = GetSystemMetrics(SM_CXSCREEN);
        let sy = GetSystemMetrics(SM_CYSCREEN);
        let x = ((sx - win_w) / 2).max(0);
        let y = ((sy - win_h) / 2).max(0);

        let scale = |px: i32| (px * dpi + 48) / 96;
        let mut ui = Box::new(SetupUi {
            hwnd,
            dpi,
            uninstall_mode,
            installed: install::is_installed(),
            finished: false,
            font_title: make_font(scale(20), FW_SEMIBOLD),
            font_body: make_font(scale(14), FW_NORMAL),
            font_small: make_font(scale(13), FW_NORMAL),
            brush_bg: CreateSolidBrush(COL_BG),
            brush_header: CreateSolidBrush(COL_HEADER),
            brush_footer: CreateSolidBrush(COL_FOOTER),
            brush_divider: CreateSolidBrush(COL_DIVIDER),
            icon_header: load_app_icon(scale(40)),
            lbl_status: null_mut(),
            chk_start_menu: null_mut(),
            chk_desktop: null_mut(),
            chk_launch: null_mut(),
            chk_remove_cfg: null_mut(),
            btn_primary: null_mut(),
            btn_secondary: null_mut(),
            btn_uninstall: null_mut(),
        });

        ui.build_controls();
        let raw = Box::into_raw(ui);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, raw as isize);

        // Position centered with exact DPI-adjusted client size.
        #[link(name = "user32")]
        extern "system" {
            fn SetWindowPos(
                hwnd: HWND,
                after: HWND,
                x: i32,
                y: i32,
                cx: i32,
                cy: i32,
                flags: UINT,
            ) -> BOOL;
        }
        const SWP_NOZORDER: UINT = 0x0004;
        SetWindowPos(hwnd, null_mut(), x, y, win_w, win_h, SWP_NOZORDER);

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
        InvalidateRect(hwnd, null(), 1);

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            if IsDialogMessageW(hwnd, &mut msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match install::parse_cli(&args) {
        CliAction::GuiInstall => run_gui(false),
        CliAction::GuiUninstall => run_gui(true),
        CliAction::SilentInstall(opts) => {
            if let Err(err) = install::install(opts) {
                message_box(null_mut(), "ClickLoop Setup", &err, MB_OK | MB_ICONERROR);
                std::process::exit(1);
            }
        }
        CliAction::SilentUninstall { remove_config } => {
            if let Err(err) = install::uninstall(remove_config) {
                message_box(null_mut(), "ClickLoop Setup", &err, MB_OK | MB_ICONERROR);
                std::process::exit(1);
            }
        }
    }
}
