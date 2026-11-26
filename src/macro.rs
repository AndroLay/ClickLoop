//! Rekam dan putar ulang aktivitas mouse (gerak kursor, klik kiri/kanan/tengah, scroll wheel)
//! dan keyboard (tekan/lepas tombol termasuk extended keys) dengan jeda waktu akurat mikrodetik
//! serta simpan/muat ke berkas `.clm`.
use std::fs;
use std::path::Path;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::clicker::{countdown_ms, Shared, PAUSE_NONE, PAUSE_WAIT_TARGET};
use crate::config::Hotkey;
use crate::guard;
use crate::keys;
use crate::wait::{StopEvent, Waiter};
use crate::win32::*;

const STACK_BYTES: usize = 64 * 1024;
/// Batas maksimum event dalam satu makro agar berkas rusak tidak menghabiskan memori.
pub const MAX_EVENTS: usize = 200_000;
/// Sampling gerak kursor saat merekam: rekam bila jarak >= MOVE_PX dan jeda >= MOVE_US.
const MOVE_PX: i32 = 3;
const MOVE_US: u64 = 8_000;
const PAUSE_SLICE: Duration = Duration::from_millis(50);
const HEADER: &str = "# ClickLoop Macro v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Move,
    LeftDown,
    LeftUp,
    RightDown,
    RightUp,
    MiddleDown,
    MiddleUp,
    Wheel(i16),
    KeyDown { vk: u16, scan: u16, extended: bool },
    KeyUp { vk: u16, scan: u16, extended: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    /// Jeda sejak event sebelumnya (mikrodetik). Event pertama selalu 0.
    pub delay_us: u64,
    pub x: i32,
    pub y: i32,
    pub action: Action,
}

/// Durasi total makro (mikrodetik).
pub fn total_us(events: &[Event]) -> u64 {
    events
        .iter()
        .fold(0u64, |acc, e| acc.saturating_add(e.delay_us))
}

/// Jumlah aksi non-gerak (klik + scroll + tekan tombol keyboard) untuk ringkasan yang ramah pengguna.
pub fn action_count(events: &[Event]) -> usize {
    events
        .iter()
        .filter(|e| {
            matches!(
                e.action,
                Action::LeftDown
                    | Action::RightDown
                    | Action::MiddleDown
                    | Action::Wheel(_)
                    | Action::KeyDown { .. }
            )
        })
        .count()
}

/// Ringkasan singkat untuk header kartu makro, misalnya "12 actions · 4.2 s" atau "No macro".
pub fn summary(events: &[Event]) -> String {
    if events.is_empty() {
        return "No macro".to_string();
    }
    let acts = action_count(events);
    let secs = total_us(events) as f64 / 1_000_000.0;
    let time = if secs < 60.0 {
        format!("{secs:.1} s")
    } else {
        let s = secs.round() as u64;
        format!("{}:{:02}", s / 60, s % 60)
    };
    if acts > 0 {
        format!(
            "{acts} {} \u{B7} {time}",
            if acts == 1 { "action" } else { "actions" }
        )
    } else {
        format!("{} moves \u{B7} {time}", events.len())
    }
}

/// Serialisasi makro ke format teks `.clm`.
pub fn to_text(events: &[Event]) -> String {
    let mut out = String::with_capacity(events.len() * 24 + 32);
    out.push_str(HEADER);
    out.push('\n');
    for e in events {
        match e.action {
            Action::Move => out.push_str(&format!("{},{},{},M\n", e.delay_us, e.x, e.y)),
            Action::LeftDown => out.push_str(&format!("{},{},{},LD\n", e.delay_us, e.x, e.y)),
            Action::LeftUp => out.push_str(&format!("{},{},{},LU\n", e.delay_us, e.x, e.y)),
            Action::RightDown => out.push_str(&format!("{},{},{},RD\n", e.delay_us, e.x, e.y)),
            Action::RightUp => out.push_str(&format!("{},{},{},RU\n", e.delay_us, e.x, e.y)),
            Action::MiddleDown => out.push_str(&format!("{},{},{},MD\n", e.delay_us, e.x, e.y)),
            Action::MiddleUp => out.push_str(&format!("{},{},{},MU\n", e.delay_us, e.x, e.y)),
            Action::Wheel(d) => out.push_str(&format!("{},{},{},W,{d}\n", e.delay_us, e.x, e.y)),
            Action::KeyDown { vk, scan, extended } => {
                out.push_str(&format!(
                    "{},0,0,KD,{vk},{scan},{}\n",
                    e.delay_us, extended as u8
                ));
            }
            Action::KeyUp { vk, scan, extended } => {
                out.push_str(&format!(
                    "{},0,0,KU,{vk},{scan},{}\n",
                    e.delay_us, extended as u8
                ));
            }
        }
    }
    out
}

/// Membaca teks makro `.clm`. Mengembalikan galat bila tidak ada event sah.
pub fn from_text(text: &str) -> Result<Vec<Event>, String> {
    let mut events = Vec::new();
    for (idx, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if events.len() >= MAX_EVENTS {
            break;
        }
        let mut parts = line.split(',');
        let delay_us = parts
            .next()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .ok_or_else(|| format!("Invalid delay on line {}.", idx + 1))?;
        let x = parts
            .next()
            .and_then(|v| v.trim().parse::<i32>().ok())
            .ok_or_else(|| format!("Invalid X on line {}.", idx + 1))?;
        let y = parts
            .next()
            .and_then(|v| v.trim().parse::<i32>().ok())
            .ok_or_else(|| format!("Invalid Y on line {}.", idx + 1))?;
        let code = parts
            .next()
            .map(str::trim)
            .ok_or_else(|| format!("Missing action on line {}.", idx + 1))?;
        let action = match code {
            "M" => Action::Move,
            "LD" => Action::LeftDown,
            "LU" => Action::LeftUp,
            "RD" => Action::RightDown,
            "RU" => Action::RightUp,
            "MD" => Action::MiddleDown,
            "MU" => Action::MiddleUp,
            "W" => {
                let d = parts
                    .next()
                    .and_then(|v| v.trim().parse::<i16>().ok())
                    .ok_or_else(|| format!("Invalid wheel delta on line {}.", idx + 1))?;
                Action::Wheel(d)
            }
            "KD" | "KU" => {
                let vk = parts
                    .next()
                    .and_then(|v| v.trim().parse::<u16>().ok())
                    .ok_or_else(|| format!("Invalid key code on line {}.", idx + 1))?;
                let scan = parts
                    .next()
                    .and_then(|v| v.trim().parse::<u16>().ok())
                    .ok_or_else(|| format!("Invalid scan code on line {}.", idx + 1))?;
                let extended = parts.next().is_some_and(|v| v.trim() == "1");
                if code == "KD" {
                    Action::KeyDown { vk, scan, extended }
                } else {
                    Action::KeyUp { vk, scan, extended }
                }
            }
            _ => return Err(format!("Unknown action \"{code}\" on line {}.", idx + 1)),
        };
        events.push(Event {
            delay_us,
            x,
            y,
            action,
        });
    }
    if events.is_empty() {
        return Err("Macro file contains no events.".to_string());
    }
    events[0].delay_us = 0;
    Ok(events)
}

pub fn save_file(path: &Path, events: &[Event]) -> std::io::Result<()> {
    fs::write(path, to_text(events))
}

pub fn load_file(path: &Path) -> Result<Vec<Event>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("Could not read macro: {e}"))?;
    from_text(&text)
}

// ---- Perekam (WH_MOUSE_LL + WH_KEYBOARD_LL pada thread UI) ----

struct RecState {
    events: Vec<Event>,
    last_time: Option<Instant>,
    last_move: Option<(i32, i32, Instant)>,
    skip_keys: [Hotkey; 3],
}

impl RecState {
    fn new(skip_keys: [Hotkey; 3]) -> Self {
        RecState {
            events: Vec::with_capacity(256),
            last_time: None,
            last_move: None,
            skip_keys,
        }
    }

    fn push(&mut self, x: i32, y: i32, action: Action, now: Instant) {
        if self.events.len() >= MAX_EVENTS {
            return;
        }
        match action {
            Action::Move => {
                if let Some((lx, ly, lt)) = self.last_move {
                    let dist = (x - lx).abs() + (y - ly).abs();
                    let dt = now.duration_since(lt).as_micros() as u64;
                    if dist < MOVE_PX || dt < MOVE_US {
                        return;
                    }
                }
                self.last_move = Some((x, y, now));
            }
            Action::KeyDown { .. } | Action::KeyUp { .. } => {}
            _ => {
                self.last_move = Some((x, y, now));
            }
        }
        let delay_us = self
            .last_time
            .map(|t| now.duration_since(t).as_micros() as u64)
            .unwrap_or(0);
        self.last_time = Some(now);
        self.events.push(Event {
            delay_us,
            x,
            y,
            action,
        });
    }
}

/// Menghapus modifier KeyDown tak berpasangan di paling belakang rekaman (mis. Ctrl/Alt/Shift dari hotkey stop).
fn trim_trailing_modifiers(events: &mut Vec<Event>) {
    while let Some(last) = events.last() {
        let is_mod_down = matches!(
            last.action,
            Action::KeyDown { vk, .. } if matches!(vk, 0x10..=0x12 | 0xA0..=0xA5)
        );
        if is_mod_down {
            events.pop();
        } else {
            break;
        }
    }
}

static REC_MOUSE_HOOK: AtomicIsize = AtomicIsize::new(0);
static REC_KEY_HOOK: AtomicIsize = AtomicIsize::new(0);
static REC_COUNT: AtomicU32 = AtomicU32::new(0);
static REC_DATA: Mutex<Option<RecState>> = Mutex::new(None);

/// Mulai merekam aktivitas mouse & keyboard global.
/// `skip_keys`: kombinasi hotkey kontrol (start, record, play) yang tidak ikut direkam.
pub unsafe fn start_recording(skip_keys: [Hotkey; 3]) -> bool {
    if REC_MOUSE_HOOK.load(Ordering::Relaxed) != 0 {
        return true;
    }
    if let Ok(mut g) = REC_DATA.lock() {
        *g = Some(RecState::new(skip_keys));
    }
    REC_COUNT.store(0, Ordering::Relaxed);
    let inst = GetModuleHandleW(null());
    let hm = SetWindowsHookExW(WH_MOUSE_LL, record_mouse_hook, inst, 0);
    if hm.is_null() {
        if let Ok(mut g) = REC_DATA.lock() {
            *g = None;
        }
        return false;
    }
    let hk = SetWindowsHookExW(WH_KEYBOARD_LL, record_key_hook, inst, 0);
    if hk.is_null() {
        UnhookWindowsHookEx(hm);
        if let Ok(mut g) = REC_DATA.lock() {
            *g = None;
        }
        return false;
    }
    REC_MOUSE_HOOK.store(hm as isize, Ordering::Relaxed);
    REC_KEY_HOOK.store(hk as isize, Ordering::Relaxed);
    true
}

/// Menghentikan perekaman dan mengembalikan seluruh event yang terekam.
pub unsafe fn stop_recording() -> Vec<Event> {
    let hm = REC_MOUSE_HOOK.swap(0, Ordering::Relaxed);
    if hm != 0 {
        UnhookWindowsHookEx(hm as HHOOK);
    }
    let hk = REC_KEY_HOOK.swap(0, Ordering::Relaxed);
    if hk != 0 {
        UnhookWindowsHookEx(hk as HHOOK);
    }
    REC_COUNT.store(0, Ordering::Relaxed);
    let mut events = REC_DATA
        .lock()
        .ok()
        .and_then(|mut g| g.take())
        .map(|s| s.events)
        .unwrap_or_default();
    trim_trailing_modifiers(&mut events);
    events
}

pub fn recorded_count() -> u32 {
    REC_COUNT.load(Ordering::Relaxed)
}

unsafe fn is_own_window(w: HWND) -> bool {
    if w.is_null() {
        return false;
    }
    let mut pid = 0u32;
    GetWindowThreadProcessId(w, &mut pid);
    pid == GetCurrentProcessId()
}

unsafe fn current_mods() -> u32 {
    let down = |vk: i32| GetAsyncKeyState(vk) as u16 & 0x8000 != 0;
    let mut m = 0;
    if down(VK_SHIFT) {
        m |= keys::SHIFT;
    }
    if down(VK_CONTROL) {
        m |= keys::CTRL;
    }
    if down(VK_MENU) {
        m |= keys::ALT;
    }
    m
}

unsafe extern "system" fn record_mouse_hook(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if code >= 0 {
        let m = &*(l as *const MSLLHOOKSTRUCT);
        if m.flags & LLMHF_INJECTED == 0 {
            let action = match w as u32 {
                WM_MOUSEMOVE => Some(Action::Move),
                WM_LBUTTONDOWN => Some(Action::LeftDown),
                WM_LBUTTONUP => Some(Action::LeftUp),
                WM_RBUTTONDOWN => Some(Action::RightDown),
                WM_RBUTTONUP => Some(Action::RightUp),
                WM_MBUTTONDOWN => Some(Action::MiddleDown),
                WM_MBUTTONUP => Some(Action::MiddleUp),
                WM_MOUSEWHEEL => {
                    let delta = ((m.mouse_data >> 16) as u16) as i16;
                    if delta != 0 {
                        Some(Action::Wheel(delta))
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some(act) = action {
                // Jangan rekam gerakan/klik di atas jendela ClickLoop sendiri (mis. saat klik tombol Stop).
                if !is_own_window(WindowFromPoint(m.pt)) {
                    if let Ok(mut g) = REC_DATA.try_lock() {
                        if let Some(state) = g.as_mut() {
                            state.push(m.pt.x, m.pt.y, act, Instant::now());
                            REC_COUNT.store(state.events.len() as u32, Ordering::Relaxed);
                        }
                    }
                }
            }
        }
    }
    CallNextHookEx(null_mut(), code, w, l)
}

unsafe extern "system" fn record_key_hook(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if code >= 0 {
        let k = &*(l as *const KBDLLHOOKSTRUCT);
        if k.flags & LLKHF_INJECTED == 0
            && k.vk_code != VK_ESCAPE
            && !is_own_window(GetForegroundWindow())
        {
            let is_down = matches!(w as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
            let is_up = matches!(w as u32, WM_KEYUP | WM_SYSKEYUP);
            if is_down || is_up {
                let mods = current_mods();
                if let Ok(mut g) = REC_DATA.try_lock() {
                    if let Some(state) = g.as_mut() {
                        let is_ctrl_hk = state.skip_keys.iter().any(|hk| {
                            hk.vk == k.vk_code && (hk.mods == mods || is_up && hk.mods == 0)
                        });
                        if !is_ctrl_hk {
                            let vk = k.vk_code as u16;
                            let scan = k.scan_code as u16;
                            let extended = (k.flags & LLKHF_EXTENDED) != 0;
                            let act = if is_down {
                                Action::KeyDown { vk, scan, extended }
                            } else {
                                Action::KeyUp { vk, scan, extended }
                            };
                            state.push(0, 0, act, Instant::now());
                            REC_COUNT.store(state.events.len() as u32, Ordering::Relaxed);
                        }
                    }
                }
            }
        }
    }
    CallNextHookEx(null_mut(), code, w, l)
}

// ---- Pemutar Makro (Thread Worker Presisi Tinggi) ----

pub struct PlayConfig {
    pub events: Vec<Event>,
    pub loop_play: bool,
    pub delay_ms: u32,
    pub max_secs: u32,
    pub focus_guard: bool,
    pub failsafe: bool,
}

pub struct Player {
    active: Arc<AtomicBool>,
    loops: Arc<AtomicU64>,
    shared: Arc<Shared>,
    worker: Option<JoinHandle<()>>,
    guard: Option<JoinHandle<()>>,
    stop_ev: HANDLE,
}

impl Player {
    pub fn new() -> Self {
        Player {
            active: Arc::new(AtomicBool::new(false)),
            loops: Arc::new(AtomicU64::new(0)),
            shared: Arc::new(Shared::default()),
            worker: None,
            guard: None,
            stop_ev: null_mut(),
        }
    }

    pub fn is_playing(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    pub fn loops(&self) -> u64 {
        self.loops.load(Ordering::Relaxed)
    }

    pub fn pause_reason(&self) -> u8 {
        self.shared.pause.load(Ordering::Relaxed)
    }

    pub fn stop_reason(&self) -> u8 {
        self.shared.stop_reason.load(Ordering::Relaxed)
    }

    pub fn countdown_ms(&self) -> u32 {
        self.shared.countdown.load(Ordering::Relaxed)
    }

    pub fn start(&mut self, cfg: PlayConfig) -> bool {
        self.stop();
        if cfg.events.is_empty() {
            return false;
        }
        let ev = unsafe { CreateEventW(null_mut(), 1, 0, null()) };
        if ev.is_null() {
            return false;
        }
        self.stop_ev = ev;
        self.loops.store(0, Ordering::Relaxed);
        let shared = Arc::new(Shared::default());
        if cfg.focus_guard {
            shared.pause.store(PAUSE_WAIT_TARGET, Ordering::Relaxed);
        }
        self.shared = shared.clone();
        self.active.store(true, Ordering::Release);
        let stop = StopEvent(ev);

        let (a, l, sh) = (self.active.clone(), self.loops.clone(), shared.clone());
        let focus = cfg.focus_guard;
        let failsafe = cfg.failsafe;
        // Kumpulkan titik mouse makro agar failsafe tidak false-trigger oleh gerak makro sendiri.
        let near: Vec<(i32, i32)> = if failsafe {
            cfg.events
                .iter()
                .filter(|e| !matches!(e.action, Action::KeyDown { .. } | Action::KeyUp { .. }))
                .map(|e| (e.x, e.y))
                .collect()
        } else {
            Vec::new()
        };

        let spawned = thread::Builder::new()
            .stack_size(STACK_BYTES)
            .spawn(move || play_run(cfg, a, l, sh, stop));
        match spawned {
            Ok(h) => self.worker = Some(h),
            Err(_) => {
                self.stop();
                return false;
            }
        }

        if focus || failsafe {
            let gcfg = guard::Config {
                focus,
                failsafe,
                near,
                radius: guard::NEAR_SLACK,
            };
            let a = self.active.clone();
            let spawned = thread::Builder::new()
                .stack_size(STACK_BYTES)
                .spawn(move || guard::run(gcfg, shared, a, stop));
            match spawned {
                Ok(h) => self.guard = Some(h),
                Err(_) => {
                    self.stop();
                    return false;
                }
            }
        }
        true
    }

    pub fn stop(&mut self) {
        self.active.store(false, Ordering::Release);
        if !self.stop_ev.is_null() {
            unsafe { SetEvent(self.stop_ev) };
        }
        if let Some(h) = self.worker.take() {
            let _ = h.join();
        }
        if let Some(h) = self.guard.take() {
            let _ = h.join();
        }
        if !self.stop_ev.is_null() {
            unsafe { CloseHandle(self.stop_ev) };
            self.stop_ev = null_mut();
        }
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}

#[derive(Default)]
struct HeldInputs {
    left: bool,
    right: bool,
    middle: bool,
    keys: Vec<(u16, u16, bool)>,
}

impl HeldInputs {
    fn key_down(&mut self, vk: u16, scan: u16, extended: bool) {
        if !self.keys.iter().any(|&(k, _, _)| k == vk) {
            self.keys.push((vk, scan, extended));
        }
    }

    fn key_up(&mut self, vk: u16) {
        self.keys.retain(|&(k, _, _)| k != vk);
    }

    unsafe fn release_all(&mut self) {
        if self.left {
            send_mouse(MOUSEEVENTF_LEFTUP, 0);
            self.left = false;
        }
        if self.right {
            send_mouse(MOUSEEVENTF_RIGHTUP, 0);
            self.right = false;
        }
        if self.middle {
            send_mouse(MOUSEEVENTF_MIDDLEUP, 0);
            self.middle = false;
        }
        for (vk, scan, extended) in self.keys.drain(..).rev() {
            send_key(vk, scan, extended, true);
        }
    }
}

unsafe fn send_mouse(flags: u32, mouse_data: u32) {
    let inp = [Input {
        ty: INPUT_MOUSE,
        u: InputUnion {
            mi: MouseInput {
                dx: 0,
                dy: 0,
                mouse_data,
                dw_flags: flags,
                time: 0,
                dw_extra_info: 0,
            },
        },
    }];
    SendInput(1, inp.as_ptr(), std::mem::size_of::<Input>() as i32);
}

unsafe fn send_key(vk: u16, scan: u16, extended: bool, up: bool) {
    let mut flags = 0u32;
    if extended {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    let inp = [Input {
        ty: INPUT_KEYBOARD,
        u: InputUnion {
            ki: KeybdInput {
                w_vk: vk,
                w_scan: scan,
                dw_flags: flags,
                time: 0,
                dw_extra_info: 0,
            },
        },
    }];
    SendInput(1, inp.as_ptr(), std::mem::size_of::<Input>() as i32);
}

fn play_run(
    cfg: PlayConfig,
    active: Arc<AtomicBool>,
    loops: Arc<AtomicU64>,
    shared: Arc<Shared>,
    stop: StopEvent,
) {
    let w = unsafe { Waiter::new(stop.0) };
    let mut held = HeldInputs::default();
    if countdown_ms(cfg.delay_ms, &w, &shared) {
        let started = Instant::now();
        let deadline =
            (cfg.max_secs > 0).then(|| started + Duration::from_secs(cfg.max_secs as u64));

        'play: loop {
            let mut lap_origin = Instant::now();
            let mut cum_us = 0u64;
            for (idx, ev) in cfg.events.iter().enumerate() {
                if idx > 0 {
                    cum_us = cum_us.saturating_add(ev.delay_us);
                }
                // Tunggu fokus kembali jika Focus guard mem-pause, sambil melepas semua tombol mouse & keyboard.
                if shared.pause.load(Ordering::Relaxed) != PAUSE_NONE {
                    unsafe { held.release_all() };
                    while shared.pause.load(Ordering::Relaxed) != PAUSE_NONE {
                        if w.stopped() || deadline.is_some_and(|d| Instant::now() >= d) {
                            break 'play;
                        }
                        let slice = Instant::now() + PAUSE_SLICE;
                        let wake = deadline.map_or(slice, |d| slice.min(d));
                        if !w.until(wake) {
                            break 'play;
                        }
                    }
                    lap_origin = Instant::now()
                        .checked_sub(Duration::from_micros(cum_us))
                        .unwrap_or_else(Instant::now);
                }
                if w.stopped() || deadline.is_some_and(|d| Instant::now() >= d) {
                    break 'play;
                }
                let target = lap_origin + Duration::from_micros(cum_us);
                let wake = deadline.map_or(target, |d| target.min(d));
                if !w.until(wake) {
                    break 'play;
                }
                if deadline.is_some_and(|d| Instant::now() >= d) {
                    break 'play;
                }
                unsafe {
                    match ev.action {
                        Action::Move => {
                            SetCursorPos(ev.x, ev.y);
                        }
                        Action::LeftDown => {
                            SetCursorPos(ev.x, ev.y);
                            send_mouse(MOUSEEVENTF_LEFTDOWN, 0);
                            held.left = true;
                        }
                        Action::LeftUp => {
                            SetCursorPos(ev.x, ev.y);
                            send_mouse(MOUSEEVENTF_LEFTUP, 0);
                            held.left = false;
                        }
                        Action::RightDown => {
                            SetCursorPos(ev.x, ev.y);
                            send_mouse(MOUSEEVENTF_RIGHTDOWN, 0);
                            held.right = true;
                        }
                        Action::RightUp => {
                            SetCursorPos(ev.x, ev.y);
                            send_mouse(MOUSEEVENTF_RIGHTUP, 0);
                            held.right = false;
                        }
                        Action::MiddleDown => {
                            SetCursorPos(ev.x, ev.y);
                            send_mouse(MOUSEEVENTF_MIDDLEDOWN, 0);
                            held.middle = true;
                        }
                        Action::MiddleUp => {
                            SetCursorPos(ev.x, ev.y);
                            send_mouse(MOUSEEVENTF_MIDDLEUP, 0);
                            held.middle = false;
                        }
                        Action::Wheel(delta) => {
                            SetCursorPos(ev.x, ev.y);
                            send_mouse(MOUSEEVENTF_WHEEL, delta as i32 as u32);
                        }
                        Action::KeyDown { vk, scan, extended } => {
                            send_key(vk, scan, extended, false);
                            held.key_down(vk, scan, extended);
                        }
                        Action::KeyUp { vk, scan, extended } => {
                            send_key(vk, scan, extended, true);
                            held.key_up(vk);
                        }
                    }
                }
            }
            loops.fetch_add(1, Ordering::Relaxed);
            if !cfg.loop_play {
                break;
            }
            // Jeda kecil antar putaran loop (16 ms) agar CPU & input queue tetap responsif meski makro sangat pendek.
            let gap = Instant::now() + Duration::from_millis(16);
            let wake = deadline.map_or(gap, |d| gap.min(d));
            if !w.until(wake) || deadline.is_some_and(|d| Instant::now() >= d) {
                break;
            }
        }
    }
    unsafe { held.release_all() };
    drop(w);
    active.store(false, Ordering::Release);
    unsafe { SetEvent(stop.0) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_serialization_mouse_and_keyboard_and_summary() {
        let events = vec![
            Event {
                delay_us: 0,
                x: 100,
                y: 200,
                action: Action::Move,
            },
            Event {
                delay_us: 15_000,
                x: 120,
                y: 220,
                action: Action::LeftDown,
            },
            Event {
                delay_us: 45_000,
                x: 120,
                y: 220,
                action: Action::LeftUp,
            },
            Event {
                delay_us: 100_000,
                x: 0,
                y: 0,
                action: Action::KeyDown {
                    vk: 0x41,
                    scan: 30,
                    extended: false,
                },
            },
            Event {
                delay_us: 40_000,
                x: 0,
                y: 0,
                action: Action::KeyUp {
                    vk: 0x41,
                    scan: 30,
                    extended: false,
                },
            },
            Event {
                delay_us: 100_000,
                x: 0,
                y: 0,
                action: Action::KeyDown {
                    vk: 0x26,
                    scan: 72,
                    extended: true,
                },
            },
            Event {
                delay_us: 50_000,
                x: 0,
                y: 0,
                action: Action::KeyUp {
                    vk: 0x26,
                    scan: 72,
                    extended: true,
                },
            },
            Event {
                delay_us: 50_000,
                x: -50,
                y: 300,
                action: Action::Wheel(-120),
            },
        ];
        let text = to_text(&events);
        let parsed = from_text(&text).unwrap();
        assert_eq!(parsed, events);
        assert_eq!(action_count(&parsed), 4);
        assert_eq!(total_us(&parsed), 400_000);
        assert_eq!(summary(&parsed), "4 actions \u{B7} 0.4 s");
    }

    #[test]
    fn from_text_rejects_empty_and_corrupt_lines() {
        assert!(from_text("# only comment\n\n").is_err());
        assert!(from_text("0,10,20,BOGUS\n").is_err());
        assert!(from_text("0,10,20,W,notanint\n").is_err());
        assert!(from_text("0,0,0,KD,bad,10,0\n").is_err());
    }

    #[test]
    fn recorder_coalesces_tiny_moves_keeps_clicks_and_trims_trailing_modifiers() {
        let mut rec = RecState::new([
            Hotkey::plain(0x75),
            Hotkey::plain(0x78),
            Hotkey::plain(0x79),
        ]);
        let t0 = Instant::now();
        rec.push(100, 100, Action::Move, t0);
        rec.push(101, 100, Action::Move, t0 + Duration::from_millis(2));
        rec.push(101, 100, Action::LeftDown, t0 + Duration::from_millis(3));
        rec.push(101, 100, Action::LeftUp, t0 + Duration::from_millis(20));
        rec.push(
            0,
            0,
            Action::KeyDown {
                vk: 0xA2,
                scan: 29,
                extended: false,
            },
            t0 + Duration::from_millis(30),
        );
        trim_trailing_modifiers(&mut rec.events);
        assert_eq!(rec.events.len(), 3);
        assert_eq!(rec.events[1].action, Action::LeftDown);
        assert_eq!(rec.events[2].action, Action::LeftUp);
    }
}
