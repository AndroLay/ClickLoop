//! Penunggu presisi yang bisa dibatalkan seketika lewat event berhenti.
//! Memakai waitable timer resolusi tinggi; tanpa polling. Setiap thread punya penunggu sendiri.
use std::ptr::{null, null_mut};
use std::thread;
use std::time::{Duration, Instant};

use crate::win32::*;

/// Handle event berhenti yang dibagi UI dan worker. Event dimiliki Runner dan ditutup setelah join.
#[derive(Clone, Copy)]
pub struct StopEvent(pub HANDLE);
unsafe impl Send for StopEvent {}

pub struct Waiter {
    stop: HANDLE,
    timer: HANDLE,
    /// Timer resolusi tinggi tidak tersedia (Windows lama): pakai timeBeginPeriod sebagai cadangan.
    coarse: bool,
}

impl Waiter {
    pub(crate) unsafe fn new(stop: HANDLE) -> Self {
        let mut timer = CreateWaitableTimerExW(
            null_mut(),
            null(),
            CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
            TIMER_ALL_ACCESS,
        );
        let coarse = timer.is_null();
        if coarse {
            timer = CreateWaitableTimerExW(null_mut(), null(), 0, TIMER_ALL_ACCESS);
            timeBeginPeriod(1);
        }
        Waiter {
            stop,
            timer,
            coarse,
        }
    }

    /// Sudah diminta berhenti? Tidak menunggu.
    pub(crate) fn stopped(&self) -> bool {
        unsafe { WaitForSingleObject(self.stop, 0) == WAIT_OBJECT_0 }
    }

    /// Menunggu sampai `target`. False jika diminta berhenti.
    pub(crate) fn until(&self, target: Instant) -> bool {
        loop {
            let now = Instant::now();
            if now >= target {
                return !self.stopped();
            }
            let left = target - now;
            let r = unsafe {
                if self.timer.is_null() {
                    let ms = left.as_micros().div_ceil(1000).min(u32::MAX as u128 - 1) as u32;
                    WaitForSingleObject(self.stop, ms)
                } else {
                    let due = -((left.as_nanos() / 100).max(1) as i64);
                    SetWaitableTimer(self.timer, &due, 0, null(), null_mut(), 0);
                    let hs = [self.stop, self.timer];
                    WaitForMultipleObjects(2, hs.as_ptr(), 0, INFINITE)
                }
            };
            if r == WAIT_OBJECT_0 {
                return false;
            }
            if r == WAIT_FAILED {
                thread::sleep(left.min(Duration::from_millis(10)));
            }
        }
    }

    /// Menunggu sampai diminta berhenti, tanpa bangun sama sekali.
    pub(crate) fn forever(&self) {
        unsafe { WaitForSingleObject(self.stop, INFINITE) };
    }
}

impl Drop for Waiter {
    fn drop(&mut self) {
        unsafe {
            if !self.timer.is_null() {
                CloseHandle(self.timer);
            }
            if self.coarse {
                timeEndPeriod(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Penunggu harus tepat waktu dan berhenti seketika saat event diset (tanpa klik sungguhan).
    #[test]
    fn waiter_is_precise_and_stops_fast() {
        unsafe {
            let ev = CreateEventW(null_mut(), 1, 0, null());
            assert!(!ev.is_null());
            let w = Waiter::new(ev);
            assert!(!w.coarse, "timer resolusi tinggi tidak tersedia");

            let mut worst = Duration::ZERO;
            for _ in 0..20 {
                let t = Instant::now();
                assert!(w.until(t + Duration::from_millis(5)));
                worst = worst.max(t.elapsed());
            }
            assert!(worst < Duration::from_millis(9), "terlambat: {worst:?}");

            let stopper = StopEvent(ev);
            let t = thread::spawn(move || {
                let s = stopper; // tangkap seluruh pembungkus Send, bukan pointer mentahnya
                thread::sleep(Duration::from_millis(30));
                SetEvent(s.0);
            });
            let begun = Instant::now();
            assert!(!w.until(begun + Duration::from_secs(5)));
            assert!(
                begun.elapsed() < Duration::from_millis(200),
                "berhenti lambat"
            );
            t.join().unwrap();
            drop(w);
            CloseHandle(ev);
        }
    }
}
