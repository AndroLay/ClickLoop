//! Core installation, update, and uninstallation logic for ClickLoop.
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::ptr::null;

use crate::win32;

pub const APP_NAME: &str = "ClickLoop";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PUBLISHER: &str = "AndroLay";
pub const APP_DESC: &str = "ClickLoop - Auto Clicker & Macro Recorder";
pub const WINDOW_CLASS: &str = "ClickLoopWindow";
pub const UNINSTALL_REG_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Uninstall\ClickLoop";

/// Release binary of ClickLoop embedded directly into `ClickLoop-Setup.exe`.
const APP_EXE: &[u8] = include_bytes!("../../target/release/clickloop.exe");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstallOptions {
    pub start_menu: bool,
    pub desktop: bool,
    pub launch: bool,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            start_menu: true,
            desktop: true,
            launch: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CliAction {
    GuiInstall,
    GuiUninstall,
    SilentInstall(InstallOptions),
    SilentUninstall { remove_config: bool },
}

pub struct InstallReport {
    pub install_dir: PathBuf,
    pub updated: bool,
}

pub fn parse_cli(args: &[String]) -> CliAction {
    let exe_stem = env::current_exe()
        .ok()
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()));
    parse_cli_with_exe(args, exe_stem.as_deref())
}

pub fn parse_cli_with_exe(args: &[String], exe_stem: Option<&str>) -> CliAction {
    let mut silent = false;
    let mut uninstall = exe_stem
        .map(|s| s.to_ascii_lowercase().starts_with("uninstall"))
        .unwrap_or(false);
    let mut remove_config = false;
    let mut opts = InstallOptions::default();

    for arg in args {
        match arg.to_ascii_lowercase().as_str() {
            "--silent" | "/s" | "-s" => silent = true,
            "--uninstall" | "/u" | "-u" => uninstall = true,
            "--no-start-menu" => opts.start_menu = false,
            "--no-desktop" => opts.desktop = false,
            "--no-launch" => opts.launch = false,
            "--remove-config" => remove_config = true,
            _ => {}
        }
    }

    match (uninstall, silent) {
        (true, true) => CliAction::SilentUninstall { remove_config },
        (true, false) => CliAction::GuiUninstall,
        (false, true) => CliAction::SilentInstall(opts),
        (false, false) => CliAction::GuiInstall,
    }
}

/// Default per-user installation directory: `%LOCALAPPDATA%\Programs\ClickLoop`.
/// Per-user location requires no UAC elevation and stays writable for `clickloop.ini`.
pub fn install_dir() -> Result<PathBuf, String> {
    if let Ok(local) = env::var("LOCALAPPDATA") {
        if !local.trim().is_empty() {
            return Ok(PathBuf::from(local).join("Programs").join(APP_NAME));
        }
    }
    if let Ok(profile) = env::var("USERPROFILE") {
        if !profile.trim().is_empty() {
            return Ok(PathBuf::from(profile)
                .join("AppData")
                .join("Local")
                .join("Programs")
                .join(APP_NAME));
        }
    }
    Err("Could not determine %LOCALAPPDATA% or %USERPROFILE% directory.".into())
}

pub fn start_menu_shortcut() -> Option<PathBuf> {
    let appdata = env::var("APPDATA").ok()?;
    if appdata.trim().is_empty() {
        return None;
    }
    Some(
        PathBuf::from(appdata)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("ClickLoop.lnk"),
    )
}

pub fn desktop_shortcut() -> Option<PathBuf> {
    let profile = env::var("USERPROFILE").ok()?;
    if profile.trim().is_empty() {
        return None;
    }
    Some(PathBuf::from(profile).join("Desktop").join("ClickLoop.lnk"))
}

pub fn is_installed() -> bool {
    install_dir()
        .map(|d| d.join("clickloop.exe").is_file())
        .unwrap_or(false)
}

pub fn uninstall_command(uninstaller: &Path, silent: bool) -> String {
    if silent {
        format!("\"{}\" --uninstall --silent", uninstaller.display())
    } else {
        format!("\"{}\" --uninstall", uninstaller.display())
    }
}

pub fn estimated_size_kb(app_bytes: usize, uninstaller_bytes: usize) -> u32 {
    let total = app_bytes.saturating_add(uninstaller_bytes);
    total.div_ceil(1024).max(1) as u32
}

/// Gracefully closes any running ClickLoop window so `clickloop.exe` can be updated or removed.
pub fn close_running_app() {
    let class = win32::wide(WINDOW_CLASS);
    let mut sent_close = false;
    for _ in 0..20 {
        let hwnd = unsafe { win32::FindWindowW(class.as_ptr(), null()) };
        if hwnd.is_null() {
            return;
        }
        if !sent_close {
            unsafe {
                win32::PostMessageW(hwnd, win32::WM_CLOSE, 0, 0);
            }
            sent_close = true;
        }
        unsafe {
            win32::Sleep(100);
        }
    }
}

fn write_with_retry(path: &Path, data: &[u8]) -> Result<(), String> {
    let mut last_err = String::new();
    for attempt in 0..6 {
        match fs::write(path, data) {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_err = e.to_string();
                if attempt < 5 {
                    unsafe {
                        win32::Sleep(150);
                    }
                }
            }
        }
    }
    Err(format!(
        "Could not write {} (close ClickLoop if it is still running): {last_err}",
        path.display()
    ))
}

fn same_path(a: &Path, b: &Path) -> bool {
    let na = a.to_string_lossy().to_ascii_lowercase();
    let nb = b.to_string_lossy().to_ascii_lowercase();
    na == nb
}

pub fn install(opts: InstallOptions) -> Result<InstallReport, String> {
    let dir = install_dir()?;
    let exe_path = dir.join("clickloop.exe");
    let uninst_path = dir.join("uninstall.exe");
    let updated = exe_path.is_file();

    close_running_app();

    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create folder {}: {e}", dir.display()))?;

    write_with_retry(&exe_path, APP_EXE)?;

    let mut uninstaller_size = 0usize;
    if let Ok(self_exe) = env::current_exe() {
        if !same_path(&self_exe, &uninst_path) {
            fs::copy(&self_exe, &uninst_path).map_err(|e| {
                format!(
                    "Failed to copy uninstaller to {}: {e}",
                    uninst_path.display()
                )
            })?;
        }
        uninstaller_size = fs::metadata(&uninst_path)
            .map(|m| m.len() as usize)
            .unwrap_or(0);
    }

    if opts.start_menu {
        if let Some(lnk) = start_menu_shortcut() {
            win32::create_shortcut(&lnk, &exe_path, &dir, APP_DESC)?;
        }
    }
    if opts.desktop {
        if let Some(lnk) = desktop_shortcut() {
            win32::create_shortcut(&lnk, &exe_path, &dir, APP_DESC)?;
        }
    }

    let icon_val = format!("{},0", exe_path.display());
    let dir_val = dir.display().to_string();
    let uninst_cmd = uninstall_command(&uninst_path, false);
    let quiet_cmd = uninstall_command(&uninst_path, true);
    let size_kb = estimated_size_kb(APP_EXE.len(), uninstaller_size);

    win32::reg_write_uninstall_entry(
        UNINSTALL_REG_KEY,
        &[
            ("DisplayName", APP_NAME),
            ("DisplayVersion", APP_VERSION),
            ("Publisher", PUBLISHER),
            ("DisplayIcon", &icon_val),
            ("InstallLocation", &dir_val),
            ("UninstallString", &uninst_cmd),
            ("QuietUninstallString", &quiet_cmd),
        ],
        &[("EstimatedSize", size_kb), ("NoModify", 1), ("NoRepair", 1)],
    )?;

    if opts.launch {
        win32::launch_exe(&exe_path, &dir)?;
    }

    Ok(InstallReport {
        install_dir: dir,
        updated,
    })
}

/// Cleanup task to run right as the uninstaller process exits (so `uninstall.exe` is unlocked).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeferredCleanup {
    pub uninstaller: PathBuf,
    pub dir: PathBuf,
}

impl DeferredCleanup {
    pub fn spawn(&self) {
        let cmd = self_cleanup_cmd(&self.uninstaller, &self.dir);
        let _ = Command::new("cmd.exe")
            .args(["/C", &cmd])
            .creation_flags_no_window()
            .spawn();
    }
}

fn self_cleanup_cmd(uninstaller: &Path, dir: &Path) -> String {
    // Retries briefly after process exit so uninstall.exe and the empty install folder are always removed.
    format!(
        "for /L %i in (1,1,6) do (ping 127.0.0.1 -n 2 >NUL & del /F /Q \"{}\" >NUL 2>&1 & rmdir \"{}\" >NUL 2>&1 & if not exist \"{}\" exit /b 0)",
        uninstaller.display(),
        dir.display(),
        uninstaller.display()
    )
}

trait CommandNoWindow {
    fn creation_flags_no_window(&mut self) -> &mut Self;
}

impl CommandNoWindow for Command {
    fn creation_flags_no_window(&mut self) -> &mut Self {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        self.creation_flags(CREATE_NO_WINDOW)
    }
}

pub fn uninstall(remove_config: bool) -> Result<Option<DeferredCleanup>, String> {
    let dir = install_dir()?;
    let exe_path = dir.join("clickloop.exe");
    let ini_path = dir.join("clickloop.ini");
    let uninst_path = dir.join("uninstall.exe");

    close_running_app();

    if let Some(lnk) = start_menu_shortcut() {
        let _ = fs::remove_file(lnk);
    }
    if let Some(lnk) = desktop_shortcut() {
        let _ = fs::remove_file(lnk);
    }

    win32::reg_delete_uninstall_entry(UNINSTALL_REG_KEY)?;

    if exe_path.exists() {
        let mut last_err = String::new();
        let mut removed = false;
        for attempt in 0..6 {
            match fs::remove_file(&exe_path) {
                Ok(()) => {
                    removed = true;
                    break;
                }
                Err(e) => {
                    last_err = e.to_string();
                    if attempt < 5 {
                        unsafe {
                            win32::Sleep(150);
                        }
                    }
                }
            }
        }
        if !removed {
            return Err(format!(
                "Could not remove {} (close ClickLoop first): {last_err}",
                exe_path.display()
            ));
        }
    }

    if remove_config && ini_path.exists() {
        let _ = fs::remove_file(&ini_path);
    }

    let running_from_uninstaller = env::current_exe()
        .map(|p| same_path(&p, &uninst_path))
        .unwrap_or(false);

    if running_from_uninstaller {
        Ok(Some(DeferredCleanup {
            uninstaller: uninst_path,
            dir,
        }))
    } else {
        if uninst_path.exists() {
            let _ = fs::remove_file(&uninst_path);
        }
        let _ = fs::remove_dir(&dir);
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_default_and_flags() {
        assert_eq!(
            parse_cli_with_exe(&[], Some("ClickLoop-Setup")),
            CliAction::GuiInstall
        );
        assert_eq!(
            parse_cli_with_exe(&[], Some("uninstall")),
            CliAction::GuiUninstall
        );
        assert_eq!(
            parse_cli_with_exe(&["--silent".into()], Some("uninstall")),
            CliAction::SilentUninstall {
                remove_config: false
            }
        );
        assert_eq!(
            parse_cli_with_exe(&["--uninstall".into()], Some("ClickLoop-Setup")),
            CliAction::GuiUninstall
        );
        assert_eq!(
            parse_cli_with_exe(
                &[
                    "--silent".into(),
                    "--no-desktop".into(),
                    "--no-launch".into()
                ],
                Some("ClickLoop-Setup"),
            ),
            CliAction::SilentInstall(InstallOptions {
                start_menu: true,
                desktop: false,
                launch: false,
            })
        );
        assert_eq!(
            parse_cli_with_exe(
                &["/U".into(), "/S".into(), "--remove-config".into()],
                Some("ClickLoop-Setup"),
            ),
            CliAction::SilentUninstall {
                remove_config: true
            }
        );
    }

    #[test]
    fn uninstall_command_quotes_path() {
        let p = Path::new(r"C:\Users\Test User\AppData\Local\Programs\ClickLoop\uninstall.exe");
        assert_eq!(
            uninstall_command(p, false),
            r#""C:\Users\Test User\AppData\Local\Programs\ClickLoop\uninstall.exe" --uninstall"#
        );
        assert_eq!(
            uninstall_command(p, true),
            r#""C:\Users\Test User\AppData\Local\Programs\ClickLoop\uninstall.exe" --uninstall --silent"#
        );

        let dir = Path::new(r"C:\Users\Test User\AppData\Local\Programs\ClickLoop");
        let cmd = self_cleanup_cmd(p, dir);
        assert!(cmd.contains("for /L %i in (1,1,6)"));
        assert!(cmd.contains(&format!("del /F /Q \"{}\"", p.display())));
        assert!(cmd.contains(&format!("rmdir \"{}\"", dir.display())));
    }

    #[test]
    fn estimated_size_rounds_up_kb() {
        assert_eq!(estimated_size_kb(0, 0), 1);
        assert_eq!(estimated_size_kb(1024, 1024), 2);
        assert_eq!(estimated_size_kb(500_000, 500_000), 977);
    }

    #[test]
    fn embedded_binary_is_valid_pe() {
        assert!(
            APP_EXE.len() > 64_000,
            "embedded clickloop.exe unexpectedly small"
        );
        assert_eq!(
            &APP_EXE[0..2],
            b"MZ",
            "embedded binary must be a valid Windows PE"
        );
    }

    #[test]
    fn com_creates_valid_shortcut_in_temp() {
        let dir = env::temp_dir();
        let lnk = dir.join(format!("clickloop-test-{}.lnk", std::process::id()));
        let target = env::current_exe().expect("current_exe");
        win32::create_shortcut(&lnk, &target, &dir, APP_DESC).expect("create_shortcut failed");
        let meta = fs::metadata(&lnk).expect("shortcut file missing");
        assert!(meta.len() > 100, "shortcut file unexpectedly empty");
        let _ = fs::remove_file(&lnk);
    }
}
