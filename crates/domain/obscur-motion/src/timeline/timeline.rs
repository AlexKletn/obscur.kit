#![doc = include_str!("timeline.doc/_module.md")]

use crate::timeline::{TimelineId, TimelinePosition, TimelineState};

#[doc = include_str!("timeline.doc/PlaybackDirection.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaybackDirection {
    #[doc = include_str!("timeline.doc/Forward.md")]
    Forward,
    #[doc = include_str!("timeline.doc/Reverse.md")]
    Reverse,
}

impl PlaybackDirection {
    #[doc = include_str!("timeline.doc/flipped.md")]
    #[must_use]
    pub const fn flipped(self) -> Self {
        match self {
            Self::Forward => Self::Reverse,
            Self::Reverse => Self::Forward,
        }
    }
}

#[doc = include_str!("timeline.doc/RepeatCount.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RepeatCount {
    #[doc = include_str!("timeline.doc/Finite.md")]
    Finite(u32),
    #[doc = include_str!("timeline.doc/Infinite.md")]
    Infinite,
}

#[doc = include_str!("timeline.doc/Timeline.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Timeline {
    #[doc = include_str!("timeline.doc/id.md")]
    id: TimelineId,
    #[doc = include_str!("timeline.doc/duration.md")]
    duration: f32,
    #[doc = include_str!("timeline.doc/state.md")]
    state: TimelineState,
    #[doc = include_str!("timeline.doc/position.md")]
    position: TimelinePosition,
    #[doc = include_str!("timeline.doc/direction.md")]
    direction: PlaybackDirection,
    #[doc = include_str!("timeline.doc/repeat.md")]
    repeat: RepeatCount,
    #[doc = include_str!("timeline.doc/alternates.md")]
    alternates: bool,
    #[doc = include_str!("timeline.doc/delay.md")]
    delay: f32,
    #[doc = include_str!("timeline.doc/delay_remaining.md")]
    delay_remaining: f32,
}

impl Timeline {
    #[doc = include_str!("timeline.doc/MIN_DURATION.md")]
    pub const MIN_DURATION: f32 = f32::EPSILON;

    #[doc = include_str!("timeline.doc/new.md")]
    #[must_use]
    pub fn new(id: TimelineId, duration: f32) -> Self {
        Self {
            id,
            duration: duration.max(Self::MIN_DURATION),
            state: TimelineState::Idle,
            position: TimelinePosition::start(),
            direction: PlaybackDirection::Forward,
            repeat: RepeatCount::Finite(1),
            alternates: false,
            delay: 0.0,
            delay_remaining: 0.0,
        }
    }

    #[doc = include_str!("timeline.doc/with_delay.md")]
    #[must_use]
    pub fn with_delay(mut self, delay: f32) -> Self {
        self.delay = if delay.is_nan() { 0.0 } else { delay };
        self
    }

    #[doc = include_str!("timeline.doc/with_repeat.md")]
    #[must_use]
    pub fn with_repeat(mut self, repeat: RepeatCount) -> Self {
        self.repeat = repeat;
        self
    }

    #[doc = include_str!("timeline.doc/with_direction.md")]
    #[must_use]
    pub fn with_direction(mut self, direction: PlaybackDirection) -> Self {
        self.direction = direction;
        self
    }

    #[doc = include_str!("timeline.doc/alternating.md")]
    #[must_use]
    pub fn alternating(mut self, alternates: bool) -> Self {
        self.alternates = alternates;
        self
    }

    #[doc = include_str!("timeline.doc/start.md")]
    pub fn start(&mut self) {
        self.position = self.iteration_start_position();
        self.delay_remaining = self.delay.max(0.0);
        self.state = if matches!(self.repeat, RepeatCount::Finite(0)) {
            TimelineState::Finished
        } else {
            TimelineState::Running
        };
        if self.delay < 0.0 {
            self.advance(-self.delay);
        }
    }

    #[doc = include_str!("timeline.doc/pause.md")]
    pub fn pause(&mut self) {
        if self.state.is_running() {
            self.state = TimelineState::Paused;
        }
    }

    #[doc = include_str!("timeline.doc/resume.md")]
    pub fn resume(&mut self) {
        if matches!(self.state, TimelineState::Paused) {
            self.state = TimelineState::Running;
        }
    }

    #[doc = include_str!("timeline.doc/advance.md")]
    pub fn advance(&mut self, delta: f32) {
        if !self.state.is_running() {
            return;
        }

        // Remaining logical time still to consume this call (always the magnitude;
        // the current `direction` decides whether it grows or shrinks `elapsed`).
        let mut remaining = delta;
        if remaining <= 0.0 {
            // A zero/negative delta does not move a forward-integrating timeline;
            // reversal is expressed via `reverse`, not a negative advance.
            return;
        }

        // A pending start delay is spent first: the timeline is running (and so
        // keeps demanding frames) but holds at the start of its first iteration
        // until the delay is exhausted. Any surplus flows into the iteration.
        if self.delay_remaining > 0.0 {
            if remaining <= self.delay_remaining {
                self.delay_remaining -= remaining;
                return;
            }
            remaining -= self.delay_remaining;
            self.delay_remaining = 0.0;
        }

        loop {
            let time_to_end = match self.direction {
                PlaybackDirection::Forward => self.duration - self.position.elapsed(),
                PlaybackDirection::Reverse => self.position.elapsed(),
            };

            if remaining < time_to_end {
                let elapsed = match self.direction {
                    PlaybackDirection::Forward => self.position.elapsed() + remaining,
                    PlaybackDirection::Reverse => self.position.elapsed() - remaining,
                };
                self.position = TimelinePosition::new(
                    elapsed,
                    (elapsed / self.duration).clamp(0.0, 1.0),
                    self.position.iteration(),
                );
                return;
            }

            remaining -= time_to_end;
            let completed = self.position.iteration().saturating_add(1);

            let exhausted = match self.repeat {
                RepeatCount::Infinite => false,
                RepeatCount::Finite(count) => completed >= count,
            };

            if exhausted {
                let (elapsed, progress) = match self.direction {
                    PlaybackDirection::Forward => (self.duration, 1.0),
                    PlaybackDirection::Reverse => (0.0, 0.0),
                };
                self.position = TimelinePosition::new(elapsed, progress, completed);
                self.state = TimelineState::Finished;
                return;
            }

            if self.alternates {
                self.direction = self.direction.flipped();
            }
            let (elapsed, progress) = match self.direction {
                PlaybackDirection::Forward => (0.0, 0.0),
                PlaybackDirection::Reverse => (self.duration, 1.0),
            };
            self.position = TimelinePosition::new(elapsed, progress, completed);
        }
    }

    #[doc = include_str!("timeline.doc/reverse.md")]
    pub fn reverse(&mut self) {
        self.direction = self.direction.flipped();
        let elapsed = (self.duration - self.position.elapsed()).clamp(0.0, self.duration);
        let progress = (1.0 - self.position.progress()).clamp(0.0, 1.0);
        self.position = TimelinePosition::new(elapsed, progress, self.position.iteration());
    }

    #[doc = include_str!("timeline.doc/seek.md")]
    pub fn seek(&mut self, progress: f32) {
        let progress = progress.clamp(0.0, 1.0);
        self.position = TimelinePosition::new(
            progress * self.duration,
            progress,
            self.position.iteration(),
        );
    }

    #[doc = include_str!("timeline.doc/iteration_start_position.md")]
    fn iteration_start_position(&self) -> TimelinePosition {
        match self.direction {
            PlaybackDirection::Forward => TimelinePosition::new(0.0, 0.0, 0),
            PlaybackDirection::Reverse => TimelinePosition::new(self.duration, 1.0, 0),
        }
    }

    #[doc = include_str!("timeline.doc/id_2.md")]
    #[must_use]
    pub const fn id(&self) -> TimelineId {
        self.id
    }

    #[doc = include_str!("timeline.doc/duration_2.md")]
    #[must_use]
    pub const fn duration(&self) -> f32 {
        self.duration
    }

    #[doc = include_str!("timeline.doc/state_2.md")]
    #[must_use]
    pub const fn state(&self) -> TimelineState {
        self.state
    }

    #[doc = include_str!("timeline.doc/position_2.md")]
    #[must_use]
    pub const fn position(&self) -> TimelinePosition {
        self.position
    }

    #[doc = include_str!("timeline.doc/progress.md")]
    #[must_use]
    pub const fn progress(&self) -> f32 {
        self.position.progress()
    }

    #[doc = include_str!("timeline.doc/is_finished.md")]
    #[must_use]
    pub const fn is_finished(&self) -> bool {
        self.state.is_finished()
    }

    #[doc = include_str!("timeline.doc/direction_2.md")]
    #[must_use]
    pub const fn direction(&self) -> PlaybackDirection {
        self.direction
    }

    #[doc = include_str!("timeline.doc/repeat_2.md")]
    #[must_use]
    pub const fn repeat(&self) -> RepeatCount {
        self.repeat
    }

    #[doc = include_str!("timeline.doc/alternates_2.md")]
    #[must_use]
    pub const fn alternates(&self) -> bool {
        self.alternates
    }

    #[doc = include_str!("timeline.doc/delay_2.md")]
    #[must_use]
    pub const fn delay(&self) -> f32 {
        self.delay
    }

    #[doc = include_str!("timeline.doc/is_delayed.md")]
    #[must_use]
    pub fn is_delayed(&self) -> bool {
        self.delay_remaining > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timeline() -> Timeline {
        Timeline::new(TimelineId::from_raw(1), 1.0)
    }

    #[test]
    fn new_has_idle_defaults() {
        let t = timeline();
        assert_eq!(t.state(), TimelineState::Idle);
        assert!(t.position().at_start());
        assert_eq!(t.direction(), PlaybackDirection::Forward);
        assert_eq!(t.repeat(), RepeatCount::Finite(1));
        assert!(!t.alternates());
        assert_eq!(t.duration(), 1.0);
    }

    #[test]
    fn non_positive_duration_is_clamped() {
        assert!(Timeline::new(TimelineId::from_raw(1), 0.0).duration() > 0.0);
        assert!(Timeline::new(TimelineId::from_raw(1), -5.0).duration() > 0.0);
        assert!(Timeline::new(TimelineId::from_raw(1), f32::NAN).duration() > 0.0);
    }

    #[test]
    fn advance_runs_to_completion() {
        let mut t = timeline();
        t.start();
        assert_eq!(t.state(), TimelineState::Running);
        t.advance(1.0);
        assert!(t.is_finished());
        assert_eq!(t.progress(), 1.0);
        assert_eq!(t.position().iteration(), 1);
    }

    #[test]
    fn advance_only_when_running() {
        let mut t = timeline();
        // Idle: no movement.
        t.advance(0.5);
        assert!(t.position().at_start());
        t.start();
        t.pause();
        t.advance(0.5);
        // Paused: still at start.
        assert!(t.position().at_start());
    }

    #[test]
    fn delta_larger_than_duration_still_finishes() {
        let mut t = timeline();
        t.start();
        t.advance(10.0);
        assert!(t.is_finished());
        assert_eq!(t.progress(), 1.0);
        assert_eq!(t.position().iteration(), 1);
    }

    #[test]
    fn finite_three_loops_three_iterations_then_finishes() {
        let mut t = timeline().with_repeat(RepeatCount::Finite(3));
        t.start();
        t.advance(0.5);
        assert_eq!(t.position().iteration(), 0);
        assert!(!t.is_finished());
        t.advance(1.0); // completes iter 0, into iter 1
        assert_eq!(t.position().iteration(), 1);
        assert!(!t.is_finished());
        t.advance(1.0); // into iter 2
        assert_eq!(t.position().iteration(), 2);
        assert!(!t.is_finished());
        t.advance(1.0); // completes the third iteration
        assert!(t.is_finished());
        assert_eq!(t.position().iteration(), 3);
        assert_eq!(t.progress(), 1.0);
    }

    #[test]
    fn infinite_never_finishes() {
        let mut t = timeline().with_repeat(RepeatCount::Infinite);
        t.start();
        for _ in 0..1_000 {
            t.advance(1.0);
            assert!(!t.is_finished());
        }
    }

    #[test]
    fn alternating_flips_direction_each_iteration() {
        let mut t = timeline()
            .with_repeat(RepeatCount::Infinite)
            .alternating(true);
        t.start();
        assert_eq!(t.direction(), PlaybackDirection::Forward);
        t.advance(1.0); // completes forward iteration -> flip to reverse
        assert_eq!(t.direction(), PlaybackDirection::Reverse);
        // Reverse iteration starts at the end (progress 1).
        assert_eq!(t.progress(), 1.0);
        t.advance(1.0); // completes reverse iteration -> flip back to forward
        assert_eq!(t.direction(), PlaybackDirection::Forward);
        assert_eq!(t.progress(), 0.0);
    }

    #[test]
    fn reverse_mirrors_position() {
        let mut t = timeline();
        t.start();
        t.advance(0.3);
        assert!((t.progress() - 0.3).abs() < 1e-6);
        t.reverse();
        assert_eq!(t.direction(), PlaybackDirection::Reverse);
        assert!((t.progress() - 0.7).abs() < 1e-6);
        assert!((t.position().elapsed() - 0.7).abs() < 1e-6);
    }

    #[test]
    fn reverse_then_advance_moves_toward_start() {
        let mut t = timeline();
        t.start();
        t.advance(0.6);
        t.reverse(); // progress now 0.4, heading down
        t.advance(0.4);
        assert!(t.is_finished());
        // A reverse iteration ends at progress 0.
        assert_eq!(t.progress(), 0.0);
    }

    #[test]
    fn single_large_delta_crosses_multiple_iterations() {
        let mut t = timeline().with_repeat(RepeatCount::Finite(5));
        t.start();
        // 2.5 iterations of a duration-1 timeline: two completed, halfway into #3.
        t.advance(2.5);
        assert_eq!(t.position().iteration(), 2);
        assert!((t.progress() - 0.5).abs() < 1e-6);
        assert!(!t.is_finished());
    }

    #[test]
    fn seek_clamps_progress() {
        let mut t = timeline();
        t.start();
        t.seek(0.25);
        assert!((t.progress() - 0.25).abs() < 1e-6);
        assert!((t.position().elapsed() - 0.25).abs() < 1e-6);
        t.seek(5.0);
        assert_eq!(t.progress(), 1.0);
        t.seek(-5.0);
        assert_eq!(t.progress(), 0.0);
    }

    #[test]
    fn start_in_reverse_begins_at_end() {
        let mut t = timeline().with_direction(PlaybackDirection::Reverse);
        t.start();
        assert_eq!(t.progress(), 1.0);
        assert!((t.position().elapsed() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn finite_zero_finishes_immediately() {
        let mut t = timeline().with_repeat(RepeatCount::Finite(0));
        t.start();
        assert!(t.is_finished());
    }

    #[test]
    fn pause_resume_round_trips() {
        let mut t = timeline();
        t.start();
        t.pause();
        assert_eq!(t.state(), TimelineState::Paused);
        t.resume();
        assert_eq!(t.state(), TimelineState::Running);
        // Resume only from paused.
        let mut idle = timeline();
        idle.resume();
        assert_eq!(idle.state(), TimelineState::Idle);
    }

    #[test]
    fn identical_deltas_are_deterministic() {
        let deltas = [0.1, 0.37, 0.02, 0.9, 1.5, 0.25];
        let mut a = timeline()
            .with_repeat(RepeatCount::Finite(3))
            .alternating(true);
        let mut b = a.clone();
        a.start();
        b.start();
        for &d in &deltas {
            a.advance(d);
            b.advance(d);
            assert_eq!(a.position(), b.position());
            assert_eq!(a.state(), b.state());
            assert_eq!(a.direction(), b.direction());
        }
    }

    #[test]
    fn a_positive_delay_holds_the_first_iteration() {
        let mut t = timeline().with_delay(0.5);
        t.start();
        assert!(t.is_delayed());
        t.advance(0.25);
        assert!(t.is_delayed(), "still inside the delay");
        assert_eq!(t.progress(), 0.0, "held at the start of iteration 1");
        // The surplus past the delay flows into the iteration.
        t.advance(0.75);
        assert!(!t.is_delayed());
        assert_eq!(t.progress(), 0.5);
    }

    #[test]
    fn a_reverse_timeline_holds_the_iteration_end_during_the_delay() {
        let mut t = timeline()
            .with_delay(1.0)
            .with_direction(PlaybackDirection::Reverse);
        t.start();
        t.advance(0.5);
        assert!(t.is_delayed());
        assert_eq!(
            t.progress(),
            1.0,
            "a reversed iteration starts at progress 1"
        );
    }

    #[test]
    fn a_negative_delay_seeks_into_the_play_cycle() {
        let mut t = Timeline::new(TimelineId::from_raw(1), 2.0).with_delay(-0.5);
        t.start();
        assert!(!t.is_delayed());
        assert_eq!(t.progress(), 0.25, "already 0.5s into a 2s cycle");
        t.advance(1.0);
        assert_eq!(t.progress(), 0.75);
    }

    #[test]
    fn a_negative_delay_past_the_active_duration_starts_finished() {
        let mut t = timeline().with_delay(-3.0);
        t.start();
        assert!(t.is_finished());
        assert_eq!(t.progress(), 1.0);
    }

    #[test]
    fn a_delayed_timeline_is_still_running() {
        let mut t = timeline().with_delay(5.0);
        t.start();
        assert_eq!(t.state(), TimelineState::Running);
        assert!(!t.is_finished());
    }
}
