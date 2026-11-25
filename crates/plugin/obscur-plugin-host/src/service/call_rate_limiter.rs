use std::{collections::VecDeque, sync::Mutex, time::Instant};

use super::ServiceCallLimit;

pub(super) struct CallRateLimiter {
    limit: ServiceCallLimit,
    attempts: Mutex<VecDeque<Instant>>,
}

impl CallRateLimiter {
    pub(super) fn new(limit: ServiceCallLimit) -> Self {
        Self {
            limit,
            attempts: Mutex::new(VecDeque::new()),
        }
    }

    pub(super) fn admit(&self) -> bool {
        let Ok(mut attempts) = self.attempts.lock() else {
            return false;
        };
        self.limit.record(&mut attempts, Instant::now())
    }
}
