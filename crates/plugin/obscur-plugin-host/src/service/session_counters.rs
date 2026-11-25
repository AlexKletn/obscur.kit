use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

#[derive(Default)]
pub(super) struct SessionCounters {
    pub(super) in_flight: AtomicUsize,
    pub(super) rate_limit_rejections: AtomicU64,
    pub(super) concurrency_limit_rejections: AtomicU64,
    pub(super) slow_calls: AtomicU64,
}

pub(super) fn increment(counter: &AtomicU64) {
    let _ = counter.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
        value.checked_add(1)
    });
}
