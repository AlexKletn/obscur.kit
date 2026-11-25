//! Read-only observation of transport call metrics.

use std::sync::{Arc, atomic::Ordering};

use super::{ServiceSessionMetrics, session_counters::SessionCounters};

/// A clonable, thread-safe view of a service session’s counters. Retaining it keeps the final counters available after session teardown.
#[derive(Clone)]
pub struct ServiceSessionObserver(pub(super) Arc<SessionCounters>);

impl ServiceSessionObserver {
    pub(super) fn new() -> Self {
        Self(Arc::default())
    }

    /// Samples each counter independently without taking a session lock.
    #[must_use]
    pub fn metrics(&self) -> ServiceSessionMetrics {
        ServiceSessionMetrics {
            in_flight: self.0.in_flight.load(Ordering::Relaxed),
            rate_limit_rejections: self.0.rate_limit_rejections.load(Ordering::Relaxed),
            concurrency_limit_rejections: self
                .0
                .concurrency_limit_rejections
                .load(Ordering::Relaxed),
            slow_calls: self.0.slow_calls.load(Ordering::Relaxed),
        }
    }
}
