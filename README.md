<p align="center">
  <img src="docs/assets/icon.png" width="84" alt="ClickLoop logo">
</p>
<h1 align="center">ClickLoop</h1>
<p align="center"><strong>One native Rust binary. High-precision auto clicker. Mouse &amp; keyboard macro recorder.</strong></p>
<p align="center">
  A lightweight, zero-dependency Windows desktop utility combining a safe 1 ms auto clicker,
  a TinyTask-style Record &amp; Playback engine (<code>.clm</code>), a Fluent Light/Dark Win32 interface,
  and a standalone per-user installer &amp; uninstaller.
</p>
<p align="center">
  <a href="https://github.com/AndroLay/ClickLoop/actions/workflows/ci.yml"><img src="https://github.com/AndroLay/ClickLoop/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI: Build, Lint &amp; Test"></a>
  <a href="https://github.com/AndroLay/ClickLoop/releases/tag/v1.0.0"><img src="https://img.shields.io/badge/Release-v1.0.0-0067C0" alt="Release v1.0.0"></a>
  <img src="https://img.shields.io/badge/Rust-2021-DEA584?logo=rust&amp;logoColor=white" alt="Rust 2021">
  <img src="https://img.shields.io/badge/Platform-Windows%2010%2F11%20x64-0078D4?logo=windows&amp;logoColor=white" alt="Windows 10/11 x64">
  <img src="https://img.shields.io/badge/Dependencies-Zero%20Crates-16A34A" alt="Zero External Crates">
  <img src="https://img.shields.io/badge/Portable-488%20KB-0F172A" alt="488 KB Portable">
  <img src="https://img.shields.io/badge/Installer-906%20KB-0F172A" alt="906 KB Setup">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-GPLv3-blue" alt="GNU GPLv3 License"></a>
</p>
<p align="center">
  <a href="#quick-start">Download &amp; install</a> ·
  <a href="#why-clickloop">Why ClickLoop</a> ·
  <a href="#inside-the-application">Screenshots</a> ·
  <a href="#features">Features</a> ·
  <a href="#default-hotkeys--cli-reference">Hotkeys &amp; CLI</a> ·
  <a href="docs/README.md">Documentation</a> ·
  <a href="CHANGELOG.md">Changelog</a>
</p>

> **Latest release: [v1.0.0](https://github.com/AndroLay/ClickLoop/releases/tag/v1.0.0).**
> Ready to use via **`ClickLoop-Setup.exe`** (installer + uninstaller, no admin required) or portable **`clickloop.exe`**.
> Both crates compile with zero external dependencies, pass `cargo clippy --all-targets -- -D warnings`, and pass all **21 unit tests** (16 app + 5 installer).

---

## Why ClickLoop?

Many Windows auto clickers are either bundled with heavy runtimes, split clicking and macro recording across separate apps, freeze the desktop when configured down to `1 ms`, or ship outdated UIs without reliable emergency stops. ClickLoop solves these problems in a single native executable built directly on Win32.

| Limitation in typical clickers | What ClickLoop provides |
| --- | --- |
| Heavy Electron / .NET runtimes or bundled adware | Pure Rust + Win32 with **zero external crates** — `488 KB` portable binary, instant startup |
| Separate tools needed for clicking vs. macro recording | Unified **Auto Clicker** and **Record & Playback** (mouse, scroll, keyboard + `.clm` files) in one balanced 2-tab window |
| `1 ms` click intervals lock up Windows or burst-flood inputs | Explicit `< 10 ms` warning + **anti-burst scheduling (`<= 5 ms`)** with kernel timer yield so UI and OS stay responsive |
| Rigid countdown presets or no delayed start | Free **`m / s / ms` delayed start** (`0–99m`, `0–99s`, `0–9999ms`) shared across clicking and macro playback |
| Hard to tell what is running when minimized | Compact **Mini mode** (`320 × 140`) showing active mode (`Clicker` / `Macro`), live status pill, tab switcher, and direct action controls |
| Runaway clicking when switching windows | Built-in **Focus guard**, **mouse-shake / top-left corner Failsafe**, and automatic release of held keys & buttons on stop |
| Messy ZIP extraction or UAC-heavy installers | Self-contained **`ClickLoop-Setup.exe`** (`906 KB`) with per-user install, Start Menu/Desktop shortcuts, and clean **Windows Settings → Apps** uninstaller |

---

## Inside the application

Actual captures from the Windows 11 release build (`v1.0.0`). Click any image to inspect at full resolution.

<table>
<tr>
<td width="50%" valign="top">
<h3>Auto Clicker (Fluent Light)</h3>
<p>Balanced 2-column layout with interval (<code>h/m/s</code> up to 99, <code>ms</code> up to 9999), random offset, press duration, click options, Hold mode, multi-point cursor targeting, delayed start, and safety toggles.</p>
<a href="docs/assets/screenshots/auto-clicker-light.png"><img src="docs/assets/screenshots/auto-clicker-light.png" width="720" alt="ClickLoop Auto Clicker tab in Fluent Light theme"></a>
</td>
<td width="50%" valign="top">
<h3>Record &amp; Playback (Fluent Dark)</h3>
<p>TinyTask-style macro capture for mouse moves, clicks, scroll wheel, and keyboard keys with <code>.clm</code> Save/Load, Loop playback, custom <code>F9</code>/<code>F10</code> global hotkeys, and microsecond replay precision.</p>
<a href="docs/assets/screenshots/record-play-dark.png"><img src="docs/assets/screenshots/record-play-dark.png" width="720" alt="ClickLoop Record and Play tab in Fluent Dark theme"></a>
</td>
</tr>
<tr>
<td width="50%" valign="top">
<h3>Compact Mini Mode</h3>
<p>Shrinks to a floating <code>320 × 140</code> bar displaying the live status pill, active mode badge (<code>Clicker</code> or <code>Macro</code>), instant tab switch, pin-on-top toggle, and mode-aware action buttons.</p>
<a href="docs/assets/screenshots/mini-mode.png"><img src="docs/assets/screenshots/mini-mode.png" width="720" alt="ClickLoop compact Mini Mode showing Macro badge and action buttons"></a>
</td>
<td width="50%" valign="top">
<h3>Native Installer &amp; Uninstaller</h3>
<p>Zero-dependency Win32 setup dialog installing into <code>%LOCALAPPDATA%\Programs\ClickLoop</code> with no UAC prompt, COM shortcuts, in-place updates, and full uninstall integration.</p>
<a href="docs/assets/screenshots/installer.png"><img src="docs/assets/screenshots/installer.png" width="720" alt="ClickLoop-Setup.exe native Windows installer dialog"></a>
</td>
</tr>
</table>

---

## Features

| Area | Capabilities | Details |
| --- | --- | --- |
| **Auto Clicker** | `h/m/s` (`0–99`), `ms` (`0–9999`), random interval jitter, press duration, Left/Right/Middle button, Single/Double/Triple click, **Hold** mode | `< 10 ms` warning + anti-burst `1 ms` safety |
| **Cursor Targeting** | Follow live cursor, lock to fixed `(X, Y)`, or cycle **2–5 picked points** with optional position jitter (`px`) | Global crosshair picker (`F8`, `Esc` to cancel) |
| **Triggers & Stop Limits** | Global toggle hotkey (`F6`), Mouse 4 / Mouse 5 hold-to-click, stop after `N` clicks or `S` seconds | Single-instance guard prevents hotkey clashes |
| **Record & Playback** | Captures cursor path (coalesced `>= 3 px` & `>= 8 ms`), mouse buttons, wheel, and keyboard down/up (scan codes + extended flags) | `.clm` save/load, Loop playback, custom `F9`/`F10` |
| **Delayed Start** | Free countdown in minutes (`0–99`), seconds (`0–99`), and milliseconds (`0–9999`) | Applies to both Auto Clicker and Macro Playback |
| **Safety Guards** | **Focus guard** (pauses when foreground window changes), **Failsafe** (rapid mouse shake or top-left `(0, 0)` corner) | Automatically releases any held buttons or keys |
| **UI & Workspace** | Fluent Light/Dark theme, DPI PerMonitorV2, balanced `580 × 540` 2-tab layout, `320 × 140` Mini mode, Always on top, System tray | Persists state in `clickloop.ini` + `.ini` profiles |
| **Installer & CLI** | Standalone `ClickLoop-Setup.exe` + `uninstall.exe`, silent flags (`--silent`, `--uninstall`), full CLI launch flags | Per-user install, registered in **Settings → Apps** |

---

## Quick start

### Option 1: Windows Installer (Recommended)

1. Download **`ClickLoop-Setup.exe`** from [Releases v1.0.0](https://github.com/AndroLay/ClickLoop/releases/tag/v1.0.0).
2. Run `ClickLoop-Setup.exe` and click **Install** (or **Update** if already installed).
3. ClickLoop is installed to `%LOCALAPPDATA%\Programs\ClickLoop`, with Start Menu/Desktop shortcuts and an uninstaller registered in **Settings → Apps**.

Silent install & uninstall flags:

```powershell
# Silent install without Desktop shortcut
.\ClickLoop-Setup.exe --silent --no-desktop

# Silent uninstall (keeps clickloop.ini unless --remove-config is passed)
& "$env:LOCALAPPDATA\Programs\ClickLoop\uninstall.exe" --silent --remove-config
```

See [docs/INSTALLER.md](docs/INSTALLER.md) for all installer and uninstaller workflows.

### Option 2: Portable executable

Download **`clickloop.exe`** from [Releases v1.0.0](https://github.com/AndroLay/ClickLoop/releases/tag/v1.0.0), place it in any writable folder, and run it directly. Settings save to `clickloop.ini` next to the executable.

### Option 3: Build from source

**Requirements**
- Windows 10 or Windows 11 (x64)
- Rust toolchain with the `x86_64-pc-windows-gnu` target
- `windres` (from MinGW-w64, e.g. MSYS2 UCRT64) on `PATH`

```powershell
git clone https://github.com/AndroLay/ClickLoop.git
cd ClickLoop

# Build portable release binary -> target\release\clickloop.exe
cargo build --release

# Build both release binary and standalone installer -> dist\ClickLoop-Setup.exe
.\scripts\build-installer.ps1
```

---

## Default hotkeys &amp; CLI reference

### Hotkeys

| Action | Default | Scope | Customizable |
| --- | --- | --- | --- |
| Start / Stop Auto Clicker | `F6` | Global | Yes (in UI) |
| Pick Cursor Position | `F8` | Global | Yes (in UI) |
| Record / Stop Macro | `F9` | Global | Yes (in UI) |
| Play / Stop Macro | `F10` | Global | Yes (in UI) |
| Cancel Picker / Hotkey Capture / Stop Macro | `Esc` | Active overlay / playback | — |

### Command-line options

| Flag | Description |
| --- | --- |
| `--interval MS` | Interval between clicks in milliseconds |
| `--limit N` | Stop after `N` clicks (`0` = no limit) |
| `--time S` | Stop after `S` seconds (`0` = no limit) |
| `--jitter MS` | Random `+/- MS` added to each interval |
| `--button NAME` | `left`, `right`, or `middle` |
| `--profile FILE` | Load an `.ini` profile before applying other CLI overrides |
| `--start` | Start Auto Clicker immediately after the window opens |
| `--help` | Display the usage dialog |

```powershell
clickloop.exe --profile farming.ini --interval 50 --limit 200 --start
```

---

## Project structure

```text
ClickLoop/
├── build.rs                      # Embeds generated .ico, PerMonitorV2/ComCtl32 manifest, VERSIONINFO
├── build/icon.rs                 # Pure-Rust anti-aliased multi-size icon renderer
├── src/
│   ├── main.rs                   # Entry point, CLI handling, single-instance mutex
│   ├── clicker.rs                # Auto clicker & Hold worker with anti-burst scheduling
│   ├── macro.rs                  # WH_MOUSE_LL + WH_KEYBOARD_LL recorder, playback, .clm I/O
│   ├── config.rs                 # Settings parser/writer (clickloop.ini) & CLI overrides
│   ├── guard.rs                  # Focus guard & mouse-shake / corner failsafe
│   ├── keys.rs                   # Virtual-key validation & human-readable key labels
│   ├── picker.rs                 # Global cursor coordinate picker overlay
│   ├── shell.rs                  # Native Windows Open/Save dialogs (.ini and .clm)
│   ├── theme.rs                  # Fluent Light/Dark palettes & GDI anti-aliased primitives
│   ├── wait.rs                   # High-resolution waitable timer + micro-spin waiter
│   ├── win32.rs                  # Zero-crate Win32 FFI bindings
│   └── ui/                       # Window state, 2-tab layout, drawing, events, mini mode, tray
├── installer/                    # Standalone zero-crate Win32 installer & uninstaller (ClickLoop-Setup.exe)
├── scripts/
│   └── build-installer.ps1       # End-to-end release + installer packaging script
├── docs/                         # User guide, architecture, installer manual, screenshots
├── CHANGELOG.md                  # Release history
└── LICENSE                       # GNU General Public License v3.0
```

---

## Verification &amp; quality gates

Run all checks locally:

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test

cargo fmt --manifest-path installer/Cargo.toml --check
cargo clippy --manifest-path installer/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path installer/Cargo.toml
```

Latest verified results (`v1.0.0`):
- **Main application (`clickloop`)**: `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, **16/16 unit tests passed**, release binary `488 KB`.
- **Installer (`clickloop-installer`)**: `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, **5/5 unit tests passed** (CLI/exe-name auto-uninstall detection, registry command quoting, size rounding, embedded PE verification, live COM `.lnk` creation), release binary `906 KB`.

---

## Notes &amp; limitations

- **Single instance:** Launching ClickLoop while already running brings the existing window to the foreground (including when minimized to the system tray) and recovers its tray icon if Windows Explorer restarts.
- **Windows UIPI:** ClickLoop runs as a standard user (`asInvoker`). Windows security blocks non-elevated applications from sending synthetic input into elevated (Administrator) windows unless ClickLoop is also started as Administrator.
- **Responsible use:** Automated mouse and keyboard input may violate the terms of service of certain online games or platforms. Use ClickLoop responsibly and only where permitted.

---

## Documentation

| Document | Contents |
| --- | --- |
| [docs/README.md](docs/README.md) | Documentation hub and reading guide |
| [docs/USER_GUIDE.md](docs/USER_GUIDE.md) | Complete guide to Auto Clicker, Record &amp; Playback, `.clm` format, profiles, CLI, and troubleshooting |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Internal design, threading model, timing precision, hook coalescing, and rendering pipeline |
| [docs/INSTALLER.md](docs/INSTALLER.md) | Interactive and silent installation, update, uninstallation, and packaging guide |
| [CHANGELOG.md](CHANGELOG.md) | Version history and release highlights |
| [LICENSE](LICENSE) | GNU General Public License v3.0 (GPL-3.0-only) |
