use std::time::Duration;

pub(super) const ADAPTATION_WINDOW: u8 = 12;

#[derive(Default)]
pub(super) struct QueueFeedback {
    samples: u8,
    pub(super) late: u8,
    pub(super) fast: u8,
    pub(super) discarded: u8,
}

impl QueueFeedback {
    pub(super) fn observe(&mut self, delay: Duration, shown: bool, interval: Duration) -> bool {
        self.samples += 1;
        if !shown {
            self.discarded += 1;
        } else if delay > 2 * interval {
            self.late += 1;
        } else if delay <= interval * 3 / 2 {
            self.fast += 1;
        }
        self.samples == ADAPTATION_WINDOW
    }
}
