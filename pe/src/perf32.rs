//! Opt-in per-guest-thread host-service timing. No guest state changes.
use std::{cell::RefCell, sync::OnceLock, time::Instant};
static ENABLED: OnceLock<bool> = OnceLock::new();
std::thread_local! {
    static TOTALS: RefCell<([u64;4],[u64;4])> = const { RefCell::new(([0;4],[0;4])) };
}
pub(super) fn start() -> Option<Instant> {
    if *ENABLED.get_or_init(|| std::env::var_os("ALKYHOL_D3D9_PROFILE").is_some()) {
        Some(Instant::now())
    } else {
        None
    }
}
pub(super) fn record(category: usize, start: Option<Instant>) {
    if let Some(start) = start {
        let elapsed = start.elapsed().as_nanos().min(u64::MAX as u128) as u64;
        TOTALS.with(|slot| {
            let mut slot = slot.borrow_mut();
            slot.0[category] = slot.0[category].saturating_add(elapsed);
            slot.1[category] = slot.1[category].saturating_add(1);
        });
    }
}
pub(super) fn take() -> ([f64; 4], [u64; 4]) {
    TOTALS.with(|slot| {
        let (nanos, calls) = std::mem::take(&mut *slot.borrow_mut());
        (nanos.map(|n| n as f64 / 1e9), calls)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshots_reset_and_workers_do_not_mix_timing_counts() {
        take();
        record(0, None);
        assert_eq!(take(), ([0.; 4], [0; 4]));
        record(1, Some(Instant::now()));
        let worker = std::thread::spawn(|| {
            assert_eq!(take(), ([0.; 4], [0; 4]));
            record(3, Some(Instant::now()));
            let (seconds, calls) = take();
            assert_eq!(calls, [0, 0, 0, 1]);
            assert!(seconds.iter().all(|n| n.is_finite() && *n >= 0.));
        });
        worker.join().unwrap();
        assert_eq!(take().1, [0, 1, 0, 0]);
        assert_eq!(take(), ([0.; 4], [0; 4]));
    }
}
