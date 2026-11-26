# ClickLoop Architecture

ClickLoop is designed around three core constraints: **zero external crates**, **predictable real-time input safety**, and **crisp native Windows ergonomics** in a sub-500 KB footprint.

---

## 1. Crate &amp; Module Map

The repository contains two standalone zero-dependency Rust crates:

1. **Main application (`Cargo.toml` → `clickloop.exe`)**
   - `build.rs` + `build/icon.rs`: Generates multi-resolution anti-aliased `.ico` assets (plus a 256×256 PNG preview), embeds a `PerMonitorV2` + `ComCtl32 v6` + `asInvoker` manifest, and embeds `VERSIONINFO` via `windres`.
   - `src/main.rs`: CLI argument parsing, `--help` dialog, and `CreateMutexW("Local\\ClickLoop.SingleInstance")` single-instance enforcement.
   - `src/win32.rs`: Typed Win32 FFI bindings (`user32`, `gdi32`, `kernel32`, `shell32`, `comdlg32`, `dwmapi`, `winmm`).
   - `src/theme.rs`: Fluent Light/Dark palettes (`Pal`), DPI-scaled fonts/brushes, and stack-allocated anti-aliased rounded-rectangle rendering (no GDI+ dependency, no large persistent bitmaps).
   - `src/ui.rs` + `src/ui/*`: Window lifecycle, balanced 2-tab layout (`layout.rs`), control construction (`create.rs`), owner-draw Fluent controls (`draw.rs`), message dispatch (`events.rs`), actions (`actions.rs`), state binding (`settings.rs`), and system tray with `TaskbarCreated` recovery (`tray.rs`).
   - `src/clicker.rs`: Auto clicker and Hold-mode worker thread.
   - `src/macro.rs`: Low-level mouse/keyboard hook recorder, `.clm` codec, and drift-free playback worker.
   - `src/wait.rs`: High-resolution interruptible `Waiter` combining Windows waitable timers (`CREATE_WAITABLE_TIMER_HIGH_RESOLUTION`) with a short final spin.
   - `src/guard.rs`: Foreground window Focus Guard and mouse-shake / top-left corner Failsafe detector.
   - `src/picker.rs`: Full-screen coordinate picker overlay using `WH_MOUSE_LL`.
   - `src/config.rs`, `src/keys.rs`, `src/shell.rs`: Settings/profile `.ini` I/O, hotkey validation, and native `GetOpenFileNameW`/`GetSaveFileNameW` dialogs (`OFN_NOCHANGEDIR`).

2. **Standalone Installer (`installer/Cargo.toml` → `ClickLoop-Setup.exe`)**
   - `installer/build.rs`: Reuses `../build/icon.rs` and embeds `ClickLoop Setup` metadata.
   - `installer/src/win32.rs`: Minimal Win32 GUI, Registry (`advapi32`), Shell (`shell32`), and raw COM vtable bindings (`IShellLinkW` + `IPersistFile`) for zero-subprocess `.lnk` creation.
   - `installer/src/install.rs`: Embedded `clickloop.exe` payload (`include_bytes!`), graceful app shutdown (`WM_CLOSE`), retry writes, shortcut generation, HKCU Uninstall registration, and deferred self-cleanup for `uninstall.exe`.
   - `installer/src/main.rs`: PerMonitorV2 DPI-scaled Win32 Setup / Uninstall UI and CLI dispatcher.

---

## 2. Core Engineering Principles

| Principle | Implementation |
| --- | --- |
| **Zero external dependencies** | Both `clickloop` and `clickloop-installer` have empty `[dependencies]` — all Win32, GDI, COM, registry, and `.ico` rendering live in clean internal modules. |
| **Re-entrancy safe Win32 state** | UI state uses `Rc<RefCell<Ui>>` (`UiCell`) with explicit `window_alive(hwnd)` guards across modal dialogs and DPI changes. |
| **Per-monitor crisp rendering** | `PerMonitorV2` DPI awareness, `AdjustWindowRectExForDpi`, custom anti-aliased rounded corners without GDI+ overhead, and clean GDI handle disposal on `WM_DPICHANGED`. |
| **Stuck-key prevention** | Macro playback tracks every synthetic mouse button and virtual key pressed and releases them automatically on stop, pause, or failsafe. |
| **Non-elevated install safety** | Installs into `%LOCALAPPDATA%\Programs\ClickLoop` (`asInvoker`) so no UAC elevation is required and `clickloop.ini` stays writable. |

---

## 3. Execution Flows (Simplified)

### Auto Clicker Loop
1. **Trigger:** User presses the global hotkey (`F6`) or holds Mouse 4 / Mouse 5.
2. **Delayed start (optional):** Runs a cancelable millisecond countdown (`m / s / ms`).
3. **Safety checks:** Verifies **Failsafe** (mouse shake or top-left corner) and **Focus guard** (pauses automatically if another window takes focus).
4. **Input &amp; timing:** Resolves the target point (live cursor, fixed `(X, Y)`, or 2–5 point sequence + optional pixel jitter), sends input via `SendInput`, and waits for the next deadline.

### Record &amp; Playback Loop
1. **Record (`F9`):** Captures mouse movement, clicks, wheel, and keyboard down/up events on an isolated hook thread, skipping ClickLoop's own window and control hotkeys, coalescing tiny moves (`>= 3 px` / `>= 8 ms`), and trimming trailing modifier keys.
2. **Store (optional):** Saves/loads macros in portable plain-text `.clm` files.
3. **Playback (`F10`):** Runs optional delayed start, replays events drift-free via `Waiter` (once or on Loop) under Focus guard &amp; Failsafe, and releases any held mouse buttons or keyboard keys immediately on pause or stop.

---

## 4. Threading &amp; State Safety Model

| Thread | Responsibility | Communication |
| --- | --- | --- |
| **UI Thread (STA)** | Runs the Win32 message loop, holds `UiCell = Rc<RefCell<Ui>>`, renders Fluent GDI UI, and polls live progress on a `250 ms` timer. | Passes immutable snapshots of settings/events to workers; reads atomic counters and status flags. |
| **Clicker / Hold Worker** | Executes delayed countdown, Focus guard / Failsafe checks, `SendInput`, and `Waiter` timing. | Controlled via `Arc<AtomicBool>` stop flag; exposes click count, countdown, and pause reason atomically. |
| **Macro Playback Worker** | Executes delayed countdown, drift-free `.clm` event replay, loop counter, and stuck-input cleanup. | Controlled via `Arc<AtomicBool>` stop flag; exposes loop count and pause state atomically. |
| **Macro Recorder Thread** | Runs a dedicated message pump for `WH_MOUSE_LL` and `WH_KEYBOARD_LL` so hook latency never hits Windows' `LowLevelHooksTimeout`. | Returns the finalized `Vec<Event>` to the UI thread on stop. |

### Re-entrancy Protection (`UiCell`)
Win32 message handlers (`wnd_proc`) are inherently re-entrant: calling `SetWindowTextW`, `DestroyWindow`, `MessageBoxW`, or native file dialogs synchronously dispatches nested messages back into `wnd_proc`.
- ClickLoop stores UI state as `Rc<RefCell<Ui>>` (`UiCell`) in `GWLP_USERDATA`.
- Every message handler clones the `Rc` before touching state, drops any `RefMut` borrow **before** invoking modal dialogs or re-entrant Win32 calls, and verifies `window_alive(hwnd)` after returning from modal flows.
- On `WM_DPICHANGED`, child windows are destroyed **before** dropping the old `Theme` GDI font/brush handles so no control ever references a freed handle.

---

## 5. Timing Precision &amp; `1 ms` Anti-Burst Scheduling

- **`Waiter` (`src/wait.rs`):** Requests `CREATE_WAITABLE_TIMER_HIGH_RESOLUTION` (Windows 10 1803+) with fallback to standard waitable timers, sleeping in the kernel until ~1–2 ms before the deadline and performing a brief bounded yield/spin for sub-millisecond accuracy while polling the `stop` atomic flag.
- **Anti-burst rule (`src/clicker.rs`):**
  - When `interval > 5 ms`: next deadline advances from the previous target (`deadline + interval`) to preserve long-term cadence without drift.
  - When `interval <= 5 ms` (such as `1 ms` clicking): `schedule_next()` advances from **completion time** (`now + interval`). If Windows pauses the thread briefly, ClickLoop never emits a sudden catch-up flood of `SendInput` events that could freeze input processing.

---

## 6. Macro Recording &amp; Replay Integrity

- **Isolated hook thread:** `WH_MOUSE_LL` and `WH_KEYBOARD_LL` run on a dedicated thread with its own message pump so hook latency stays tiny and never trips Windows' `LowLevelHooksTimeout`.
- **Event filtering &amp; coalescing:**
  - Ignores events with `LLMHF_INJECTED` / `LLKHF_INJECTED` flags (prevents feedback loops).
  - Ignores mouse events occurring inside ClickLoop's own window rectangle.
  - Skips configured control hotkeys (`rec_key`, `play_key`, `Esc`) and trims trailing modifier key-downs captured right as the user hits the stop hotkey.
  - Coalesces mouse move samples (`>= 3 px` Euclidean delta and `>= 8 ms` interval) to keep `.clm` files compact and smooth.
- **Held-input tracking:** Playback maintains the set of currently pressed synthetic mouse buttons and virtual keys (`pressed_keys`). On stop, completion, failsafe trigger, or Focus Guard pause, every held button/key receives a matching key-up/button-up `SendInput` event so the user is never left with a stuck `Shift`, `Ctrl`, or mouse button.

---

## 7. Single-Instance &amp; System Tray Recovery

- A named mutex (`Local\ClickLoop.SingleInstance`) ensures only one ClickLoop instance registers global hotkeys.
- A second launch locates `"ClickLoopWindow"` via `FindWindowW`, posts custom message `WM_RESTORE_APP`, and calls `SetForegroundWindow` — restoring the app seamlessly even when hidden in the system tray.
- ClickLoop registers `"TaskbarCreated"` at startup and re-adds its tray icon automatically if Windows Explorer restarts while minimized to tray.
