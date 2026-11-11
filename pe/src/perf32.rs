//! Opt-in per-guest-thread host-service timing. No guest state changes.
use std::{
    cell::RefCell,
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
    thread::ThreadId,
    time::Instant,
};
static ENABLED: OnceLock<bool> = OnceLock::new();
static LEASES: Mutex<Vec<(ThreadId, u64, u64)>> = Mutex::new(Vec::new());
std::thread_local! {
    static TOTALS: RefCell<([u64;4],[u64;4])> = const { RefCell::new(([0;4],[0;4])) };
    static APIS: RefCell<BTreeMap<u32, (u64, u64)>> = const { RefCell::new(BTreeMap::new()) };
}
pub(super) fn enabled() -> bool {
    *ENABLED.get_or_init(|| std::env::var_os("ALKYHOL_PE32_PROFILE_SERVICES").is_some())
}
pub(super) fn start() -> Option<Instant> {
    if enabled() {
        Some(Instant::now())
    } else {
        None
    }
}
pub(super) fn record(category: usize, api: Option<u32>, start: Option<Instant>) {
    if let Some(start) = start {
        let elapsed = start.elapsed().as_nanos().min(u64::MAX as u128) as u64;
        TOTALS.with(|slot| {
            let mut slot = slot.borrow_mut();
            slot.0[category] = slot.0[category].saturating_add(elapsed);
            slot.1[category] = slot.1[category].saturating_add(1);
        });
        if let Some(api) = api {
            APIS.with(|slot| {
                let mut slot = slot.borrow_mut();
                let (nanos, calls) = slot.entry(api).or_default();
                *nanos = nanos.saturating_add(elapsed);
                *calls = calls.saturating_add(1);
            });
        }
    }
}
pub(super) fn take_apis() -> Vec<(u32, f64, u64)> {
    APIS.with(|slot| {
        let entries = std::mem::take(&mut *slot.borrow_mut());
        let mut entries: Vec<_> = entries
            .into_iter()
            .map(|(id, (nanos, calls))| (id, nanos as f64 / 1e9, calls))
            .collect();
        entries.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        entries
    })
}
pub(super) fn record_lease(start: Option<Instant>) {
    if let Some(start) = start {
        let elapsed = start.elapsed().as_nanos().min(u64::MAX as u128) as u64;
        let id = std::thread::current().id();
        let mut leases = LEASES.lock().unwrap();
        if let Some((_, nanos, count)) = leases.iter_mut().find(|(thread, _, _)| *thread == id) {
            *nanos = nanos.saturating_add(elapsed);
            *count = count.saturating_add(1);
        } else {
            leases.push((id, elapsed, 1));
        }
    }
}
pub(super) fn take_leases() -> Vec<(ThreadId, f64, u64)> {
    let mut leases: Vec<_> = std::mem::take(&mut *LEASES.lock().unwrap())
        .into_iter()
        .map(|(id, nanos, count)| (id, nanos as f64 / 1e9, count))
        .collect();
    leases.sort_by(|a, b| b.1.total_cmp(&a.1));
    leases
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
        take_apis();
        record(0, Some(1), None);
        assert_eq!(take(), ([0.; 4], [0; 4]));
        record(1, Some(42), Some(Instant::now()));
        let worker = std::thread::spawn(|| {
            assert_eq!(take(), ([0.; 4], [0; 4]));
            assert!(take_apis().is_empty());
            record(3, Some(9), Some(Instant::now()));
            let (seconds, calls) = take();
            assert_eq!(calls, [0, 0, 0, 1]);
            assert!(seconds.iter().all(|n| n.is_finite() && *n >= 0.));
            assert_eq!(
                take_apis()
                    .iter()
                    .map(|&(id, _, count)| (id, count))
                    .collect::<Vec<_>>(),
                [(9, 1)]
            );
        });
        worker.join().unwrap();
        assert_eq!(take().1, [0, 1, 0, 0]);
        assert_eq!(take(), ([0.; 4], [0; 4]));
        let entries = take_apis();
        assert_eq!(entries.len(), 1);
        assert_eq!((entries[0].0, entries[0].2), (42, 1));
        assert!(take_apis().is_empty());
        take_leases();
        record_lease(Some(Instant::now()));
        let current = std::thread::current().id();
        let other = std::thread::spawn(|| {
            record_lease(Some(Instant::now()));
            std::thread::current().id()
        })
        .join()
        .unwrap();
        let leases: Vec<_> = take_leases()
            .into_iter()
            .filter(|(thread, _, _)| [current, other].contains(thread))
            .collect();
        assert_eq!(leases.len(), 2);
        for (thread, seconds, count) in leases {
            assert!([current, other].contains(&thread));
            assert!(seconds.is_finite() && seconds >= 0.);
            assert_eq!(count, 1);
        }
        assert!(
            take_leases()
                .iter()
                .all(|(thread, _, _)| ![current, other].contains(thread))
        );
    }
}
