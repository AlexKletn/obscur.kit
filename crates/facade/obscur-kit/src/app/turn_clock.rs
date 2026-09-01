#![doc = include_str!("turn_clock.doc/_module.md")]

#[doc = include_str!("turn_clock.doc/TurnClock.md")]
#[derive(Debug)]
pub(super) struct TurnClock {
    #[doc = include_str!("turn_clock.doc/last.md")]
    last: Option<std::time::Instant>,
    #[doc = include_str!("turn_clock.doc/ceiling.md")]
    ceiling: f32,
}

impl TurnClock {
    #[doc = include_str!("turn_clock.doc/new.md")]
    pub(super) const fn new(ceiling: f32) -> Self {
        Self {
            last: None,
            ceiling,
        }
    }

    #[doc = include_str!("turn_clock.doc/elapsed.md")]
    pub(super) fn elapsed(&mut self, now: std::time::Instant) -> f32 {
        // `replace`, in one step, is the whole of this type. Reading the mark
        // and moving it are the same event: a caller that could do one without
        // the other is a caller that can hand the same interval out twice.
        let Some(previous) = self.last.replace(now) else {
            return 0.0;
        };
        now.duration_since(previous).as_secs_f32().min(self.ceiling)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn the_first_reading_advances_no_time() {
        let mut clock = TurnClock::new(0.1);
        assert!((clock.elapsed(Instant::now()) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn consecutive_turns_report_the_interval_between_them() {
        // THE defect this exists for. The desktop-surface loop advanced motion
        // on every turn but moved its mark only on the turns that built a
        // frame, so two turns 10 ms apart handed the motion clock 10 ms and
        // then 20 ms — 30 ms of animation for 20 ms of wall clock, and worse
        // the longer a frame took to be buildable.
        let base = Instant::now();
        let mut clock = TurnClock::new(1.0);
        let _ = clock.elapsed(base);
        let first = clock.elapsed(base + Duration::from_millis(10));
        let second = clock.elapsed(base + Duration::from_millis(20));
        assert!((first - 0.010).abs() < 1e-6, "first turn: {first}");
        assert!((second - 0.010).abs() < 1e-6, "second turn: {second}");
        assert!(
            (first + second - 0.020).abs() < 1e-6,
            "the two turns together must be the wall clock between them"
        );
    }

    #[test]
    fn many_turns_between_two_frames_sum_to_the_wall_clock() {
        // The loop turns for a bounded wait, for an input burst and for a
        // resource that landed; only some of those turns end in a frame. What
        // motion is told across all of them must still be the time that passed.
        let base = Instant::now();
        let mut clock = TurnClock::new(1.0);
        let _ = clock.elapsed(base);
        let mut total = 0.0_f32;
        for turn in 1..=8 {
            total += clock.elapsed(base + Duration::from_millis(turn * 5));
        }
        assert!((total - 0.040).abs() < 1e-6, "total: {total}");
    }

    #[test]
    fn a_long_sleep_is_clamped_once_and_not_on_every_turn_after_it() {
        // A surface that was idle for a minute must not resume by jumping every
        // animation forward by a minute — and must not then jump it again on
        // the next turn, which is what a mark that had not moved would do.
        let base = Instant::now();
        let mut clock = TurnClock::new(0.1);
        let _ = clock.elapsed(base);
        let woken = clock.elapsed(base + Duration::from_secs(60));
        let after = clock.elapsed(base + Duration::from_secs(60) + Duration::from_millis(8));
        assert!((woken - 0.1).abs() < 1e-6, "clamped: {woken}");
        assert!((after - 0.008).abs() < 1e-6, "the turn after: {after}");
    }
}
