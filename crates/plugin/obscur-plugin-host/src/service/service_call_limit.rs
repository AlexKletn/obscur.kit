use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

/// Maximum transport call attempts per rolling time window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceCallLimit {
    max_calls: usize,
    window: Duration,
}

impl ServiceCallLimit {
    /// Returns no limit configuration for a zero-length window.
    /// A zero call budget denies every transport call.
    #[must_use]
    pub fn new(max_calls: usize, window: Duration) -> Option<Self> {
        (!window.is_zero()).then_some(Self { max_calls, window })
    }
}

impl ServiceCallLimit {
    pub(super) fn record(self, attempts: &mut VecDeque<Instant>, now: Instant) -> bool {
        while attempts
            .front()
            .is_some_and(|previous| now.duration_since(*previous) >= self.window)
        {
            attempts.pop_front();
        }
        if attempts.len() >= self.max_calls {
            return false;
        }
        attempts.push_back(now);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restores_only_the_capacity_whose_attempts_left_the_rolling_window() {
        let window = Duration::from_secs(4);
        let limit = ServiceCallLimit::new(2, window).unwrap();
        let now = Instant::now();
        let mut attempts = VecDeque::new();
        assert!(limit.record(&mut attempts, now));
        assert!(limit.record(&mut attempts, now + Duration::from_secs(1)));
        assert!(!limit.record(&mut attempts, now + Duration::from_secs(3)));
        assert!(limit.record(&mut attempts, now + window));
        assert!(!limit.record(&mut attempts, now + window));
        assert!(limit.record(&mut attempts, now + Duration::from_secs(5)));
    }
}
