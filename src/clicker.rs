//! Mesin klik: satu thread worker, SendInput, jitter (interval dan posisi), sequence, hitung mundur, dan jeda.
//! Penjaga fokus dan failsafe ada di guard.rs dan berjalan di thread sendiri, hanya bila diaktifkan.
//! Thread hanya hidup saat berjalan dan stack-nya dibatasi 64 KB agar RAM tetap kecil.
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::config::Settings;
use crate::guard;
use crate::wait::{StopEvent, Waiter};
use crate::win32::*;

const MULTI_GAP_MS: u64 = 40;
const STACK_BYTES: usize = 64 * 1024;
/// Batas atas kompensasi ketertinggalan jadwal (ms) untuk interval normal.
const MAX_LAG_MS: u64 = 500;
/// Di bawah atau sama dengan interval ini (ms), catch-up burst dinonaktifkan agar input queue dan CPU tetap aman.
const NO_BURST_MS: u64 = 5;
const MIN_INTERVAL_MS: i64 = 1;
/// Saat jeda, kondisi dicek ulang tiap irisan ini.
const PAUSE_SLICE: Duration = Duration::from_millis(50);

/// Nilai `Shared::pause`. Selain PAUSE_NONE, klik dan tahan menunggu.
pub const PAUSE_NONE: u8 = 0;
pub const PAUSE_WAIT_TARGET: u8 = 1;
pub const PAUSE_FOCUS_LOST: u8 = 2;
/// Nilai `Shared::stop_reason` bila failsafe menghentikan klik.
pub const STOP_CORNER: u8 = 1;
pub const STOP_SHAKE: u8 = 2;

/// Keadaan yang dibagi UI, worker, dan penjaga. Dibuat ulang setiap kali mulai.
#[derive(Default)]
pub struct Shared {
    /// PAUSE_*: jeda fokus atau menunggu jendela target.
    pub pause: AtomicU8,
    /// STOP_*: alasan berhenti oleh failsafe; 0 bila berhenti biasa.
    pub stop_reason: AtomicU8,
    /// Sisa hitung mundur sebelum mulai (milidetik); 0 bila tidak sedang menghitung.
    pub countdown: AtomicU32,
}

pub struct Runner {
    active: Arc<AtomicBool>,
    count: Arc<AtomicU64>,
    shared: Arc<Shared>,
    worker: Option<JoinHandle<()>>,
    guard: Option<JoinHandle<()>>,
    stop_ev: HANDLE,
}

impl Runner {
    pub fn new() -> Self {
        Runner {
            active: Arc::new(AtomicBool::new(false)),
            count: Arc::new(AtomicU64::new(0)),
            shared: Arc::new(Shared::default()),
            worker: None,
            guard: None,
            stop_ev: null_mut(),
        }
    }

    pub fn is_running(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    pub fn clicks(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
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

    /// Mulai worker baru (dan penjaga bila diaktifkan). False jika event atau thread gagal dibuat.
    pub fn start(&mut self, cfg: Settings) -> bool {
        self.stop();
        let ev = unsafe { CreateEventW(null_mut(), 1, 0, null()) };
        if ev.is_null() {
            return false;
        }
        self.stop_ev = ev;
        self.count.store(0, Ordering::Relaxed);
        let shared = Arc::new(Shared::default());
        if cfg.focus_guard {
            shared.pause.store(PAUSE_WAIT_TARGET, Ordering::Relaxed);
        }
        self.shared = shared.clone();
        self.active.store(true, Ordering::Relaxed);
        let stop = StopEvent(ev);

        let (a, c, sh) = (self.active.clone(), self.count.clone(), shared.clone());
        let worker_cfg = cfg.clone();
        let spawned = thread::Builder::new()
            .stack_size(STACK_BYTES)
            .spawn(move || run(worker_cfg, a, c, sh, stop));
        match spawned {
            Ok(h) => self.worker = Some(h),
            Err(_) => {
                self.stop();
                return false;
            }
        }

        if cfg.focus_guard || cfg.failsafe {
            let gcfg = guard::Config {
                focus: cfg.focus_guard,
                failsafe: cfg.failsafe,
                near: cfg.aim_points().unwrap_or_default(),
                radius: cfg.pos_jitter as i32 + guard::NEAR_SLACK,
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

    /// Menghentikan worker dan penjaga, lalu menunggu selesai. Aman dipanggil saat tidak berjalan.
    pub fn stop(&mut self) {
        self.active.store(false, Ordering::Relaxed);
        if !self.stop_ev.is_null() {
            unsafe { SetEvent(self.stop_ev) };
        }
        if let Some(h) = self.worker.take() {
            let _ = h.join();
        }
        if let Some(h) = self.guard.take() {
            let _ = h.join();
        }
        self.close_event();
    }

    fn close_event(&mut self) {
        if !self.stop_ev.is_null() {
            unsafe { CloseHandle(self.stop_ev) };
            self.stop_ev = null_mut();
        }
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn cursor_pos() -> (i32, i32) {
    let mut p = POINT::default();
    unsafe { GetCursorPos(&mut p) };
    (p.x, p.y)
}

fn run(
    cfg: Settings,
    active: Arc<AtomicBool>,
    count: Arc<AtomicU64>,
    shared: Arc<Shared>,
    stop: StopEvent,
) {
    let w = unsafe { Waiter::new(stop.0) };
    if countdown_ms(cfg.delay_ms, &w, &shared) {
        if cfg.hold {
            hold_run(&cfg, &w, &shared);
        } else {
            click_run(&cfg, &w, &count, &shared);
        }
    }
    drop(w);
    active.store(false, Ordering::Release);
    // Selesai dengan sendirinya (batas tercapai): bangunkan penjaga agar ikut berhenti.
    unsafe { SetEvent(stop.0) };
}

/// Hitung mundur presisi milidetik sebelum mulai. False bila diminta berhenti selama hitung mundur.
pub(crate) fn countdown_ms(delay_ms: u32, w: &Waiter, shared: &Shared) -> bool {
    if delay_ms == 0 {
        return true;
    }
    let end = Instant::now() + Duration::from_millis(delay_ms as u64);
    loop {
        let now = Instant::now();
        if now >= end {
            shared.countdown.store(0, Ordering::Relaxed);
            return !w.stopped();
        }
        let left = end - now;
        shared.countdown.store(
            left.as_millis().min(u32::MAX as u128).max(1) as u32,
            Ordering::Relaxed,
        );
        let slice = left.min(Duration::from_millis(100));
        if !w.until(now + slice) {
            shared.countdown.store(0, Ordering::Relaxed);
            return false;
        }
    }
}

const VK_XBUTTON1: i32 = 0x05;
const VK_XBUTTON2: i32 = 0x06;
/// Jeda pengecekan tombol pemicu mode hold.
const TRIGGER_POLL: Duration = Duration::from_millis(15);

/// Tombol pemicu (Mouse4 atau Mouse5) sedang ditekan?
pub fn trigger_down(cfg: &Settings) -> bool {
    let vk = if cfg.trigger_btn == 0 {
        VK_XBUTTON1
    } else {
        VK_XBUTTON2
    };
    unsafe { GetAsyncKeyState(vk) as u16 & 0x8000 != 0 }
}

/// Menunggu jadwal klik berikutnya. Mode hold memeriksa tombol tiap TRIGGER_POLL dan kembali
/// lebih awal bila dilepas. False bila diminta berhenti.
fn wait_click(cfg: &Settings, w: &Waiter, target: Instant) -> bool {
    if !cfg.trigger_hold {
        return w.until(target);
    }
    loop {
        if !trigger_down(cfg) {
            return !w.stopped();
        }
        let now = Instant::now();
        if now >= target {
            return !w.stopped();
        }
        if !w.until(target.min(now + TRIGGER_POLL)) {
            return false;
        }
    }
}

/// Mode klik berulang: interval, jitter, posisi (tetap atau sequence), tipe klik, dan batas.
fn click_run(cfg: &Settings, w: &Waiter, count: &AtomicU64, shared: &Shared) {
    let started = Instant::now();
    let deadline = (cfg.max_secs > 0).then(|| started + Duration::from_secs(cfg.max_secs as u64));
    let mut rng = Rng::seeded();
    let (down, up) = buttons(cfg.button);
    let base = cfg.interval_ms() as i64;
    let limit = cfg.max_clicks as u64;
    let per_action = cfg.clicks_per_action();
    let aim = cfg.aim_points();
    let mut at = 0usize;
    let mut next = Instant::now();

    'run: loop {
        // Pastikan semua syarat aktif (belum berhenti, belum lewat batas waktu, fokus sah, dan pemicu ditahan)
        // terpenuhi bersamaan tepat sebelum menembakkan klik.
        let mut waited = false;
        loop {
            if w.stopped() || deadline.is_some_and(|d| Instant::now() >= d) {
                break 'run;
            }
            if shared.pause.load(Ordering::Relaxed) != PAUSE_NONE {
                waited = true;
                let slice = Instant::now() + PAUSE_SLICE;
                let wake = deadline.map_or(slice, |d| slice.min(d));
                if !w.until(wake) {
                    break 'run;
                }
                continue;
            }
            if cfg.trigger_hold && !trigger_down(cfg) {
                waited = true;
                let slice = Instant::now() + TRIGGER_POLL;
                let wake = deadline.map_or(slice, |d| slice.min(d));
                if !w.until(wake) {
                    break 'run;
                }
                continue;
            }
            break;
        }
        if waited {
            // Setelah jeda atau melepas tombol pemicu, jadwal dimulai dari sekarang agar tidak ada semburan klik.
            next = Instant::now();
        }
        if let Some(pts) = &aim {
            let (px, py) = pts[at % pts.len()];
            at = at.wrapping_add(1);
            let (jx, jy) = offset(&mut rng, cfg.pos_jitter);
            unsafe { SetCursorPos(px + jx, py + jy) };
        }
        for i in 0..per_action {
            if i > 0 && !w.until(Instant::now() + Duration::from_millis(MULTI_GAP_MS)) {
                break 'run;
            }
            let ok = unsafe { click(down, up, cfg.press_ms, w) };
            let total = count.fetch_add(1, Ordering::Relaxed) + 1;
            if !ok || (limit > 0 && total >= limit) {
                break 'run;
            }
        }
        let jitter = if cfg.jitter > 0 {
            (rng.next_u64() % (2 * cfg.jitter as u64 + 1)) as i64 - cfg.jitter as i64
        } else {
            0
        };
        // Minimal 1 ms: interval 0 tetap secepat mungkin tanpa memakan satu inti CPU penuh.
        let step_ms = (base + jitter).max(MIN_INTERVAL_MS) as u64;
        let step = Duration::from_millis(step_ms);
        let now = Instant::now();
        next = schedule_next(next, now, step, step_ms);
        // Batas waktu yang jatuh sebelum klik berikutnya langsung dihormati.
        let target = deadline.map_or(next, |d| next.min(d));
        if !wait_click(cfg, w, target) {
            break;
        }
    }
}

/// Menghitung jadwal klik berikutnya secara aman terhadap ketertinggalan (anti-burst).
/// Pada interval ultra-rendah (<= 5 ms, termasuk 1 ms), bila jadwal sudah lewat `now` maka
/// dijadwalkan minimal `step` dari `now` sehingga worker selalu menyerahkan CPU ke kernel via timer
/// dan tidak membanjiri input queue Windows dengan burst `SendInput` beruntun.
fn schedule_next(prev: Instant, now: Instant, step: Duration, step_ms: u64) -> Instant {
    let candidate = prev + step;
    if step_ms <= NO_BURST_MS {
        if candidate <= now {
            now + step
        } else {
            candidate
        }
    } else {
        let max_lag = Duration::from_millis(step_ms.saturating_mul(3).min(MAX_LAG_MS));
        if candidate + max_lag < now {
            now
        } else {
            candidate
        }
    }
}

/// Mode tahan: tombol ditekan dan ditahan. Dengan penjaga fokus, tombol dilepas saat jeda dan
/// ditekan lagi saat jendela target kembali. Tombol selalu dilepas di akhir agar tidak macet.
fn hold_run(cfg: &Settings, w: &Waiter, shared: &Shared) {
    let (down, up) = buttons(cfg.button);
    let size = std::mem::size_of::<Input>() as i32;
    let end = (cfg.max_secs > 0).then(|| Instant::now() + Duration::from_secs(cfg.max_secs as u64));
    let mut rng = Rng::seeded();
    let mut held = false;
    loop {
        let blocked = shared.pause.load(Ordering::Relaxed) != PAUSE_NONE
            || (cfg.trigger_hold && !trigger_down(cfg));
        if blocked == held {
            if !held && cfg.fixed {
                let (jx, jy) = offset(&mut rng, cfg.pos_jitter);
                unsafe { SetCursorPos(cfg.x + jx, cfg.y + jy) };
            }
            let one = [mk(if held { up } else { down })];
            unsafe { SendInput(1, one.as_ptr(), size) };
            held = !held;
        }
        // Bangun berkala hanya bila ada penjaga fokus atau pemicu; tanpa keduanya, tidur sampai selesai.
        let watch = cfg.focus_guard || cfg.trigger_hold;
        let slice = if cfg.trigger_hold {
            TRIGGER_POLL
        } else {
            PAUSE_SLICE
        };
        let wake = match end {
            None if !watch => {
                w.forever();
                break;
            }
            None => Instant::now() + slice,
            Some(e) if !watch => e,
            Some(e) => e.min(Instant::now() + slice),
        };
        if !w.until(wake) {
            break;
        }
        if end.is_some_and(|e| Instant::now() >= e) {
            break;
        }
    }
    if held {
        let one = [mk(up)];
        unsafe { SendInput(1, one.as_ptr(), size) };
    }
}

fn buttons(t: u32) -> (u32, u32) {
    match t {
        1 => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        2 => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
        _ => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
    }
}

fn mk(flags: u32) -> Input {
    Input {
        ty: INPUT_MOUSE,
        u: InputUnion {
            mi: MouseInput {
                dx: 0,
                dy: 0,
                mouse_data: 0,
                dw_flags: flags,
                time: 0,
                dw_extra_info: 0,
            },
        },
    }
}

/// Satu klik. Tanpa tahan, down+up dikirim sekaligus dalam satu SendInput.
/// Dengan tahan, tombol selalu dilepas walau berhenti di tengah agar tidak macet.
/// Mengembalikan false jika dihentikan selama menahan.
unsafe fn click(down: u32, up: u32, hold_ms: u32, w: &Waiter) -> bool {
    let size = std::mem::size_of::<Input>() as i32;
    if hold_ms == 0 {
        let pair = [mk(down), mk(up)];
        SendInput(2, pair.as_ptr(), size);
        return true;
    }
    let press = [mk(down)];
    SendInput(1, press.as_ptr(), size);
    let ok = w.until(Instant::now() + Duration::from_millis(hold_ms as u64));
    let release = [mk(up)];
    SendInput(1, release.as_ptr(), size);
    ok
}

/// Geseran acak (px) per sumbu dalam [-r, r].
fn offset(rng: &mut Rng, r: u32) -> (i32, i32) {
    if r == 0 {
        return (0, 0);
    }
    let m = 2 * r as u64 + 1;
    let x = (rng.next_u64() % m) as i64 - r as i64;
    let y = (rng.next_u64() % m) as i64 - r as i64;
    (x as i32, y as i32)
}

/// xorshift64, cukup untuk jitter. Tanpa crate rand.
struct Rng(u64);

impl Rng {
    fn seeded() -> Self {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as u64)
            .unwrap_or(1);
        Rng(n | 1)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_next_prevents_burst_at_one_ms_and_keeps_precision() {
        let base = Instant::now();
        let one_ms = Duration::from_millis(1);

        // Saat tidak tertinggal: tetap jadwalkan tepat prev + 1 ms.
        let on_time = schedule_next(base, base, one_ms, 1);
        assert_eq!(on_time, base + one_ms);

        // Saat terjadi lag OS 50 ms pada interval 1 ms: jangan pernah menjadwalkan di masa lalu (anti-burst).
        let after_stall = base + Duration::from_millis(50);
        let next = schedule_next(base, after_stall, one_ms, 1);
        assert!(next > after_stall);
        assert_eq!(next, after_stall + one_ms);

        // Pada interval normal (100 ms): lag kecil masih dikompensasi, lag besar di-reset ke now.
        let step_100 = Duration::from_millis(100);
        let small_lag_now = base + Duration::from_millis(180);
        assert_eq!(
            schedule_next(base, small_lag_now, step_100, 100),
            base + step_100
        );
        let huge_lag_now = base + Duration::from_millis(500);
        assert_eq!(
            schedule_next(base, huge_lag_now, step_100, 100),
            huge_lag_now
        );
    }
}
