use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

use super::session_counters::{SessionCounters, increment};

// These atomics track metrics only and do not publish session state.
pub(super) struct CallPermit<'a> {
    counters: &'a SessionCounters,
    timing: Option<(Instant, Duration)>,
}

impl<'a> CallPermit<'a> {
    pub(super) fn acquire(
        counters: &'a SessionCounters,
        limit: usize,
        slow_call_threshold: Option<Duration>,
    ) -> Option<Self> {
        counters
            .in_flight
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |count| {
                (count < limit).then(|| count + 1)
            })
            .inspect_err(|_| increment(&counters.concurrency_limit_rejections))
            .ok()?;
        Some(Self {
            counters,
            timing: slow_call_threshold.map(|threshold| (Instant::now(), threshold)),
        })
    }
}

impl Drop for CallPermit<'_> {
    fn drop(&mut self) {
        if self
            .timing
            .is_some_and(|(start, threshold)| start.elapsed() >= threshold)
        {
            increment(&self.counters.slow_calls);
        }
        self.counters.in_flight.fetch_sub(1, Ordering::Relaxed);
    }
}
