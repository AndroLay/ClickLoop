# Installer &amp; Uninstaller Guide

**ClickLoop** ships with a standalone, zero-dependency Windows installer (`ClickLoop-Setup.exe`) that embeds the release `clickloop.exe` binary and doubles as its own uninstaller (`uninstall.exe`).

---

## 1. Interactive Installation &amp; Update

1. Run `ClickLoop-Setup.exe`.
2. Review the install location (`%LOCALAPPDATA%\Programs\ClickLoop`) and options:
   - **Create Start Menu shortcut** (default: checked)
   - **Create Desktop shortcut** (default: checked)
   - **Launch ClickLoop after installation** (default: checked)
3. Click **Install** (or **Update** if ClickLoop is already installed).

### What the installer does
- Gracefully closes any running ClickLoop window (`FindWindowW("ClickLoopWindow")` + `WM_CLOSE`) so `clickloop.exe` can be replaced cleanly.
- Writes `clickloop.exe` and copies itself as `uninstall.exe` inside `%LOCALAPPDATA%\Programs\ClickLoop`.
- Creates `.lnk` shortcuts natively via Windows COM (`IShellLinkW` + `IPersistFile`) — no PowerShell or script subprocesses.
- Registers ClickLoop in **Windows Settings → Apps → Installed apps** under `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\ClickLoop`.
- Requires **no Administrator / UAC elevation**, ensuring `%LOCALAPPDATA%\Programs\ClickLoop\clickloop.ini` stays writable by the app.

---

## 2. Uninstalling ClickLoop

ClickLoop can be uninstalled cleanly in any of three ways:

1. **Windows Settings:** Open **Settings → Apps → Installed apps**, find **ClickLoop**, and click **Uninstall**.
2. **Direct Uninstaller:** Double-click `%LOCALAPPDATA%\Programs\ClickLoop\uninstall.exe` (automatically detects its filename and launches directly in Uninstall mode).
3. **Setup Wizard:** Run `ClickLoop-Setup.exe` and click **Uninstall**.

### What the uninstaller removes
- Closes any running ClickLoop instance.
- Deletes Start Menu (`%APPDATA%\Microsoft\Windows\Start Menu\Programs\ClickLoop.lnk`) and Desktop (`%USERPROFILE%\Desktop\ClickLoop.lnk`) shortcuts.
- Deletes `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\ClickLoop`.
- Deletes `clickloop.exe`.
- Keeps `clickloop.ini` by default so your preferences survive reinstalls — check **Also delete saved settings (`clickloop.ini`)** (or pass `--remove-config`) to remove it too.
- Schedules a silent post-exit cleanup when launched from `uninstall.exe` itself so `uninstall.exe` and the empty `ClickLoop` folder are removed completely.

---

## 3. Silent / Scripted CLI Flags

Both `ClickLoop-Setup.exe` and `uninstall.exe` support non-interactive flags for automation:

| Flag | Alias | Effect |
| --- | --- | --- |
| `--silent` | `/S`, `-s` | Run without showing the GUI window |
| `--uninstall` | `/U`, `-u` | Run in uninstall mode (automatic when executable name starts with `uninstall`) |
| `--no-start-menu` | — | Skip Start Menu shortcut (silent install) |
| `--no-desktop` | — | Skip Desktop shortcut (silent install) |
| `--no-launch` | — | Do not launch ClickLoop after silent install |
| `--remove-config` | — | Delete `clickloop.ini` during silent uninstall |

### Examples

```powershell
# Silent install with Start Menu shortcut only, do not launch immediately
.\ClickLoop-Setup.exe --silent --no-desktop --no-launch

# Silent uninstall preserving clickloop.ini
& "$env:LOCALAPPDATA\Programs\ClickLoop\uninstall.exe" --silent

# Complete silent uninstall including clickloop.ini
& "$env:LOCALAPPDATA\Programs\ClickLoop\uninstall.exe" --silent --remove-config
```

---

## 4. Installed File &amp; Registry Layout

### Files
```text
%LOCALAPPDATA%\Programs\ClickLoop\
├── clickloop.exe       # Main application binary
├── uninstall.exe       # Built-in uninstaller
└── clickloop.ini       # Created automatically by ClickLoop on first save
```

### Shortcuts
- `%APPDATA%\Microsoft\Windows\Start Menu\Programs\ClickLoop.lnk`
- `%USERPROFILE%\Desktop\ClickLoop.lnk` (optional)

### Registry (`HKCU`)
Key: `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\ClickLoop`
- `DisplayName`: `ClickLoop`
- `DisplayVersion`: `1.0.0`
- `Publisher`: `AndroLay`
- `DisplayIcon`: `<InstallLocation>\clickloop.exe,0`
- `InstallLocation`: `%LOCALAPPDATA%\Programs\ClickLoop`
- `UninstallString`: `"<InstallLocation>\uninstall.exe" --uninstall`
- `QuietUninstallString`: `"<InstallLocation>\uninstall.exe" --uninstall --silent`
- `EstimatedSize`: size in KB (`REG_DWORD`)
- `NoModify` = `1`, `NoRepair` = `1`

---

## 5. Building the Installer from Source

Use the packaging script from the repository root:

```powershell
.\scripts\build-installer.ps1
```

This builds `target\release\clickloop.exe`, embeds it into `installer\target\release\ClickLoop-Setup.exe`, and copies both release deliverables into `dist/`:
- `dist\ClickLoop-Setup.exe`
- `dist\clickloop.exe`
