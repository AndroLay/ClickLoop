# ClickLoop User Guide

This guide covers every control, workflow, file format, and safety feature in **ClickLoop v1.0.0**.

---

## 1. Interface Overview

ClickLoop uses a balanced 2-tab Fluent window (`580 × 540`) with Light and Dark themes:

- **Header bar:** App icon &amp; title, mode switcher (**Auto Clicker** | **Record &amp; Play**), live status pill (`Ready`, `Countdown`, `Running`, `Recording`, `Playing`, `Paused`), theme toggle (Light/Dark), and **Always on top** pin button.
- **Footer bar:** Live status/warning message on the left; **Mini mode** button and an adaptive primary action button on the right (`Start (F6)` in Auto Clicker tab, or `Record (F9)` / `Play (F10)` / `Stop` in Record &amp; Play tab).

---

## 2. Auto Clicker Mode

### Click Interval &amp; 1 ms Safety
- Configure **Hours**, **Minutes**, **Seconds** (`0–99` each), and **Milliseconds** (`0–9999`).
- The live click-rate readout updates automatically (for example `≈ 10.0 clicks/s` at `100 ms`).
- **Low-interval warning (`< 10 ms`):** Setting an interval below `10 ms` highlights the rate label and status bar in red to warn of aggressive click speeds.
- **Anti-burst protection (`<= 5 ms`):** At ultra-low intervals (including `1 ms`), ClickLoop yields to the kernel timer after each click instead of accumulating catch-up bursts, keeping Windows responsive.
- **Random offset (`± ms`):** Adds random timing variance around the base interval to avoid rigid cadence.
- **Press duration (`ms`):** Controls how long the mouse button remains held down per click (`0` = instant down/up).

### Click Options &amp; Hold Mode
- **Mouse button:** `Left`, `Right`, or `Middle`.
- **Click type:** `Single`, `Double`, or `Triple`.
- **Hold button down:** Keeps the selected mouse button continuously pressed until stopped (disables click-interval and click-count fields while active).

### Cursor Position
- **Cursor:** Clicks wherever your pointer currently is.
- **Fixed:** Locks clicks to a single `(X, Y)` screen coordinate. Click **Pick** (or press `F8`) to sample any coordinate on screen; press `Esc` or click **Cancel** on the overlay to abort.
- **Sequence:** Cycles through **2 to 5** picked `(X, Y)` points in order. Use **Pick** repeatedly to add points and **Clear points** to reset.
- **Position jitter (`± px`):** Randomly offsets each click within `±N` pixels around the target point.

### Stop After &amp; Trigger
- **Stop after:** Limit a run by total **Clicks** and/or **Time (sec)** (`0` = no limit).
- **Trigger:**
  - **Hotkey:** Press `F6` (customizable) to toggle start/stop.
  - **Mouse 4 / Mouse 5:** Clicks only while you hold the chosen side mouse button.

---

## 3. Record &amp; Playback Mode (Macros)

Switch to the **Record &amp; Play** tab to record and replay multi-step mouse and keyboard routines.

### Recording a Macro
1. Click **Record (F9)** or press your global Record hotkey (`F9` by default) from any window.
2. Perform your mouse movements, left/right/middle clicks, scroll wheel actions, and keyboard input.
   - Interactions inside the ClickLoop window itself and the control hotkeys (`F9`, `F10`, `Esc`) are excluded automatically.
   - Tiny pointer noise is coalesced (`>= 3 px` and `>= 8 ms`) so recordings stay compact and accurate.
3. Press `F9`, `Esc`, or click **Stop** to finish recording. Any trailing modifier keys pressed while stopping are trimmed automatically.

### Playing Back a Macro
1. Click **Play (F10)** or press `F10` globally.
2. Enable **Loop playback** to repeat continuously until stopped (or until **Stop after → Time** expires).
3. Configure **Delayed start (`m / s / ms`)** if you want a countdown before playback begins so you can focus your target window.
4. Press `F10` or `Esc` at any time to stop immediately. All synthetically held mouse buttons and keyboard keys are released automatically.

### Saving &amp; Loading `.clm` Files
- Click **Save** to export the current macro to a `.clm` file, or **Load** to import a previously saved `.clm` macro.
- `.clm` files are plain-text UTF-8 files with microsecond timestamps, making them portable and inspectable:

```text
CLM1
0,M,640,420
125000,LD,640,420
185000,LU,640,420
310000,KD,65,30,0
370000,KU,65,30,0
520000,W,640,420,-120
```

| Event code | Meaning | Fields |
| --- | --- | --- |
| `M` | Cursor move | `us,M,x,y` |
| `LD` / `LU` | Left button down / up | `us,LD,x,y` / `us,LU,x,y` |
| `RD` / `RU` | Right button down / up | `us,RD,x,y` / `us,RU,x,y` |
| `MD` / `MU` | Middle button down / up | `us,MD,x,y` / `us,MU,x,y` |
| `W` | Mouse wheel scroll | `us,W,x,y,delta` |
| `KD` / `KU` | Keyboard key down / up | `us,KD,vk,scan,ext` / `us,KU,vk,scan,ext` |

---

## 4. Shared Controls, Mini Mode &amp; Safety

### Delayed Start
Available in both tabs (`0–99 m`, `0–99 s`, `0–9999 ms`; all `0` = immediate start). Shows a live countdown in the status pill and can be canceled at any moment before execution starts.

### Custom Hotkeys
Click any hotkey box (**Start / stop**, **Pick position**, **Record / stop**, **Play / stop**) and press the desired key combination:
- Function keys (`F1–F24`), navigation/numpad keys, and modifier combinations (`Ctrl`/`Alt`/`Shift` + key) are supported.
- Plain unmodified typing letters are rejected to prevent interfering with normal typing.
- Duplicate hotkeys are rejected automatically. Press `Esc` to cancel capture.

### Safety Guards
- **Focus guard:** Remembers the active foreground window when clicking/playback starts. If you switch to another window, ClickLoop pauses automatically and resumes when you return.
- **Failsafe:** Immediately stops clicking or macro playback if you rapidly shake the mouse back and forth or slam the cursor into the top-left corner `(0, 0)` of the primary monitor.

### Mini Mode &amp; System Tray
- **Mini mode (`320 × 140`):** Displays the live status pill, an active mode badge (`Clicker` or `Macro`), a quick tab switcher, **Always on top** toggle, **Expand** button, and direct start/record/play controls for the active mode.
- **Minimize to tray:** Hides ClickLoop to the Windows notification area when minimized. Double-click the tray icon or right-click for quick actions (`Show`, `Start/Stop`, `Exit`).

---

## 5. Settings, Profiles &amp; CLI

### Automatic Persistence (`clickloop.ini`)
All settings, hotkeys, theme preference, and window position are saved automatically to `clickloop.ini` beside `clickloop.exe`.

### Profiles (`.ini`)
Use **Save profile** and **Load profile** to maintain reusable presets for different apps or workflows. Loading a profile preserves your current Light/Dark theme and window position.

### Command Line Reference

```powershell
clickloop.exe [--profile FILE] [--interval MS] [--limit N] [--time S] [--jitter MS] [--button left|right|middle] [--start] [--help]
```

Options applied after `--profile` override values from the loaded profile.

---

## 6. Troubleshooting

| Symptom | Cause &amp; Solution |
| --- | --- |
| Clicks or macro keys do nothing inside an elevated app (Task Manager, Admin terminal, some games) | **Windows UIPI** blocks non-elevated apps from sending synthetic input into Administrator windows. Right-click ClickLoop and choose **Run as administrator** when automating elevated targets. |
| Settings are not remembered after closing ClickLoop | `clickloop.ini` is stored next to `clickloop.exe`. Use the installer (`%LOCALAPPDATA%\Programs\ClickLoop`) or move the portable `.exe` out of read-only directories like `Program Files`. |
| Launching ClickLoop a second time does not open a second window | Expected behavior — ClickLoop enforces a single instance (`Local\ClickLoop.SingleInstance`) so global hotkeys never conflict, and brings the existing window (including from system tray) to the front. |
| Hotkey registration fails | Another application on Windows already holds that global hotkey. Pick a different hotkey or add a modifier (`Ctrl`/`Alt`/`Shift`). |
