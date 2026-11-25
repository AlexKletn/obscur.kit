//! Restart backoff for a single plugin instance or installation, as scoped by the host.

use std::time::{Duration, Instant};

/// Tracks consecutive failures and blocks restart after the configured limit.
#[derive(Debug)]
pub struct CrashRecovery {
    first_delay: Duration,
    maximum_delay: Duration,
    failure_limit: u32,
    failures: u32,
    last_failure: Option<Instant>,
    delay: Duration,
}

impl CrashRecovery {
    /// Creates a policy with an initial delay, maximum delay, and failure limit.
    #[must_use]
    pub fn new(first_delay: Duration, maximum_delay: Duration, failure_limit: u32) -> Self {
        Self {
            first_delay,
            maximum_delay: maximum_delay.max(first_delay),
            failure_limit: failure_limit.max(1),
            failures: 0,
            last_failure: None,
            delay: Duration::ZERO,
        }
    }

    /// Records one failure at the supplied monotonic time and updates the retry delay.
    pub fn record_failure(&mut self, now: Instant) {
        self.delay = if self.failures == 0 {
            self.first_delay
        } else {
            self.delay.saturating_mul(2).min(self.maximum_delay)
        };
        self.failures = self.failures.saturating_add(1);
        self.last_failure = Some(now);
    }

    /// Returns the remaining delay before retry. Zero permits immediate retry. A missing value means the host must explicitly reset the block.
    #[must_use]
    pub fn retry_after(&self, now: Instant) -> Option<Duration> {
        if self.failures >= self.failure_limit {
            return None;
        }
        Some(self.last_failure.map_or(Duration::ZERO, |last| {
            self.delay
                .saturating_sub(now.saturating_duration_since(last))
        }))
    }

    /// Clears the backoff after stable operation or an explicit host reset.
    pub fn reset(&mut self) {
        self.failures = 0;
        self.last_failure = None;
        self.delay = Duration::ZERO;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retries_back_off_then_require_a_reset() {
        let start = Instant::now();
        let second = Duration::from_secs(1);
        let mut recovery = CrashRecovery::new(second, second * 3, 4);
        assert_eq!(recovery.retry_after(start), Some(Duration::ZERO));
        for delay in [1, 2, 3] {
            recovery.record_failure(start);
            assert_eq!(recovery.retry_after(start), Some(second * delay));
            assert_eq!(
                recovery.retry_after(start + second * delay),
                Some(Duration::ZERO)
            );
        }
        recovery.record_failure(start);
        assert_eq!(recovery.retry_after(start + second * 100), None);
        recovery.reset();
        assert_eq!(recovery.retry_after(start), Some(Duration::ZERO));
    }
}
