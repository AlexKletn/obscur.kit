#![doc = include_str!("timer.doc/_module.md")]

use std::time::Duration;

#[doc = include_str!("timer.doc/Timer.md")]
pub trait Timer {
    #[doc = include_str!("timer.doc/schedule.md")]
    fn schedule(&self, delay: Duration);
}
