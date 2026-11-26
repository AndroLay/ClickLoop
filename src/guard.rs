//! Penjaga klik: jeda saat jendela target kehilangan fokus, dan berhenti saat failsafe terpicu.
//! Failsafe: kursor di pojok kiri-atas layar, atau kursor digoyang cepat bolak-balik.
//! Sampel kursor di dekat titik klik yang dikonfigurasi diabaikan, agar klik sendiri tidak dihitung goyangan.
//! Thread ini hanya hidup saat klik berjalan dan fitur penjaga aktif; interval 60 ms (fokus) atau 16 ms (failsafe).
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::clicker::{
    Shared, PAUSE_FOCUS_LOST, PAUSE_NONE, PAUSE_WAIT_TARGET, STOP_CORNER, STOP_SHAKE,
};
use crate::wait::{StopEvent, Waiter};
use crate::win32::*;

const SM_XVIRTUALSCREEN: i32 = 76;
const SM_YVIRTUALSCREEN: i32 = 77;
const TICK_FOCUS: Duration = Duration::from_millis(60);
const TICK_FAILSAFE: Duration = Duration::from_millis(16);
/// Jarak (px) dari pojok yang dihitung sebagai pojok.
const CORNER_PX: i32 = 2;
/// Goyang: minimal balik arah sebanyak SHAKE_TURNS dalam SHAKE_WINDOW dengan total gerak SHAKE_PATH px.
const SHAKE_WINDOW: Duration = Duration::from_millis(500);
const SHAKE_TURNS: u32 = 4;
const SHAKE_PATH: u32 = 800;
/// Langkah per sampel di bawah ini dianggap getaran kecil dan diabaikan.
const SHAKE_STEP: i32 = 6;
/// Jarak tambahan dari titik klik agar gerak kursor oleh klik sendiri (dan jitter) tidak terdeteksi.
pub const NEAR_SLACK: i32 = 12;

pub struct Config {
    pub focus: bool,
    pub failsafe: bool,
    /// Titik klik yang dikonfigurasi; sampel kursor di dekatnya diabaikan.
    pub near: Vec<(i32, i32)>,
    /// Radius (px) di sekitar titik klik, termasuk jitter posisi.
    pub radius: i32,
}

/// Jalankan penjaga sampai diminta berhenti atau failsafe terpicu.
/// Saat failsafe terpicu: stop_reason diisi, active dimatikan, dan event berhenti diset.
pub fn run(cfg: Config, shared: Arc<Shared>, active: Arc<AtomicBool>, stop: StopEvent) {
    let w = unsafe { Waiter::new(stop.0) };
    let period = if cfg.failsafe {
        TICK_FAILSAFE
    } else {
        TICK_FOCUS
    };
    let (vx, vy) = unsafe {
        (
            GetSystemMetrics(SM_XVIRTUALSCREEN),
            GetSystemMetrics(SM_YVIRTUALSCREEN),
        )
    };
    // Bila titik klik berada di pojok, pojok tidak dipakai sebagai failsafe (kursor memang di sana saat klik).
    let corner_used = cfg
        .near
        .iter()
        .any(|&(x, y)| x <= vx + CORNER_PX + cfg.radius && y <= vy + CORNER_PX + cfg.radius);
    // Target = jendela eksternal pertama yang aktif setelah mulai. Jendela ClickLoop tidak pernah jadi target.
    let mut target: HWND = null_mut();
    let mut shake = Shake::default();

    loop {
        if cfg.focus {
            let fg = unsafe { GetForegroundWindow() };
            if target.is_null() && !fg.is_null() && !is_own(fg) {
                target = fg;
            }
            let pause = if target.is_null() {
                PAUSE_WAIT_TARGET
            } else if fg == target {
                PAUSE_NONE
            } else {
                PAUSE_FOCUS_LOST
            };
            shared.pause.store(pause, Ordering::Relaxed);
        }
        if cfg.failsafe {
            let mut p = POINT::default();
            unsafe { GetCursorPos(&mut p) };
            let near = cfg
                .near
                .iter()
                .any(|&(px, py)| (px - p.x).abs() <= cfg.radius && (py - p.y).abs() <= cfg.radius);
            let corner = !corner_used && p.x <= vx + CORNER_PX && p.y <= vy + CORNER_PX;
            let shaken = if near {
                shake.reset();
                false
            } else {
                shake.feed((p.x, p.y), Instant::now())
            };
            let reason = if corner {
                STOP_CORNER
            } else if shaken {
                STOP_SHAKE
            } else {
                0
            };
            if reason != 0 {
                shared.stop_reason.store(reason, Ordering::Relaxed);
                active.store(false, Ordering::Release);
                unsafe { SetEvent(stop.0) };
                break;
            }
        }
        if !w.until(Instant::now() + period) {
            break;
        }
    }
}

/// Jendela milik proses ini sendiri (UI ClickLoop).
fn is_own(h: HWND) -> bool {
    let mut pid = 0u32;
    unsafe {
        GetWindowThreadProcessId(h, &mut pid);
        pid == GetCurrentProcessId()
    }
}

/// Pendeteksi goyang: hitung balik arah per sumbu dan total gerak dalam satu jendela waktu.
#[derive(Default)]
struct Shake {
    last: Option<(i32, i32)>,
    window_start: Option<Instant>,
    /// Arah langkah terakhir per sumbu (-1, 0, atau 1).
    sign: (i32, i32),
    turns: u32,
    path: u32,
}

impl Shake {
    fn reset(&mut self) {
        *self = Shake::default();
    }

    /// Menambah sampel. True bila pola goyang terdeteksi.
    fn feed(&mut self, p: (i32, i32), now: Instant) -> bool {
        let Some((lx, ly)) = self.last.replace(p) else {
            return false;
        };
        let (dx, dy) = (p.0 - lx, p.1 - ly);
        let fresh = matches!(self.window_start, Some(t) if now.duration_since(t) <= SHAKE_WINDOW);
        if !fresh {
            self.window_start = Some(now);
            self.sign = (0, 0);
            self.turns = 0;
            self.path = 0;
        }
        self.path = self
            .path
            .saturating_add(dx.unsigned_abs() + dy.unsigned_abs());
        if dx.abs() >= SHAKE_STEP {
            let s = dx.signum();
            if self.sign.0 != 0 && s != self.sign.0 {
                self.turns += 1;
            }
            self.sign.0 = s;
        }
        if dy.abs() >= SHAKE_STEP {
            let s = dy.signum();
            if self.sign.1 != 0 && s != self.sign.1 {
                self.turns += 1;
            }
            self.sign.1 = s;
        }
        self.turns >= SHAKE_TURNS && self.path >= SHAKE_PATH
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shake_ignores_straight_movement_and_triggers_on_rapid_back_and_forth() {
        let mut s = Shake::default();
        let t0 = Instant::now();

        // Gerak lurus jauh tanpa balik arah tidak boleh memicu failsafe.
        for i in 0..10 {
            assert!(!s.feed((i * 150, 100), t0 + Duration::from_millis(i as u64 * 20)));
        }

        // Goyangan cepat bolak-balik (> 4 belokan dan >= 800 px dalam <= 500 ms) harus memicu.
        s.reset();
        let pts = [
            (500, 500),
            (720, 500),
            (500, 500),
            (720, 500),
            (500, 500),
            (720, 500),
        ];
        let mut triggered = false;
        for (i, &p) in pts.iter().enumerate() {
            if s.feed(p, t0 + Duration::from_millis(i as u64 * 40)) {
                triggered = true;
            }
        }
        assert!(triggered);
    }

    #[test]
    fn shake_resets_after_window_expires() {
        let mut s = Shake::default();
        let t0 = Instant::now();
        assert!(!s.feed((100, 100), t0));
        assert!(!s.feed((350, 100), t0 + Duration::from_millis(50)));
        assert!(!s.feed((100, 100), t0 + Duration::from_millis(100)));
        assert!(!s.feed((350, 100), t0 + Duration::from_millis(150)));
        // Jeda melebihi SHAKE_WINDOW: hitungan mulai ulang sehingga belokan berikutnya belum memicu.
        assert!(!s.feed((100, 100), t0 + Duration::from_millis(800)));
    }
}
