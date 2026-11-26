# Changelog

All notable changes to **ClickLoop** are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-11-27

### Added
- **Auto Clicker engine**:
  - Configurable click interval (`hours`, `minutes`, `seconds` up to `99`; `milliseconds` up to `9999`), random interval jitter (`± ms`), and press duration.
  - Low-interval warning (`< 10 ms`) and anti-burst `<= 5 ms` scheduling so `1 ms/click` runs safely without flooding the Windows input queue.
  - Left/Right/Middle button selection, Single/Double/Triple clicks, and continuous **Hold** mode.
  - Cursor targeting modes: Follow cursor, Fixed `(X, Y)`, and multi-point **Sequence** (2–5 points) with optional position jitter (`± px`) and global crosshair coordinate picker (`F8`).
  - Global toggle hotkey (`F6`) and Mouse 4 / Mouse 5 hold-to-click triggers, plus click-count and elapsed-time stop limits.
- **Record &amp; Playback macro engine**:
  - Low-level mouse (`WH_MOUSE_LL`) and keyboard (`WH_KEYBOARD_LL`) recording capturing movement (coalesced `>= 3 px` / `>= 8 ms`), clicks, scroll wheel, and key down/up events with scan codes and extended flags.
  - High-precision drift-free playback with single-run and **Loop** modes, customizable global hotkeys (`F9` Record, `F10` Play), and automatic release of held buttons/keys on stop or pause.
  - Portable plain-text `.clm` Save and Load support via native Windows file dialogs.
- **Shared Controls, UI &amp; Safety**:
  - Customizable `m / s / ms` **Delayed start** shared by Auto Clicker and Macro Playback.
  - **Focus guard** and mouse-shake / top-left corner **Failsafe**.
  - Balanced 2-tab Fluent Light/Dark UI (`580 × 540`), DPI `PerMonitorV2` scaling, Always on top, Minimize to system tray (with `TaskbarCreated` recovery), single-instance restore, `.ini` profile import/export, and CLI flags.
  - Informative compact **Mini mode** (`320 × 140`) showing active mode (`Clicker` / `Macro`), live status pill, tab switcher, and mode-aware action buttons.
- **Standalone Windows Installer &amp; Uninstaller (`ClickLoop-Setup.exe`)**:
  - Pure-Rust zero-dependency setup wizard embedding the release `clickloop.exe`.
  - Per-user installation to `%LOCALAPPDATA%\Programs\ClickLoop` (no UAC prompt), native COM `.lnk` shortcuts, Windows **Settings → Apps** uninstall registration, standalone `uninstall.exe`, and silent CLI flags (`--silent`, `--uninstall`, `--remove-config`).
- **Documentation**:
  - Comprehensive README with real application screenshots and Mermaid architecture/workflow diagrams, plus `docs/USER_GUIDE.md`, `docs/ARCHITECTURE.md`, and `docs/INSTALLER.md`.
