//! The [`Timeline`] deterministic playable timeline (spec §4.17 "motion engine").

use crate::timeline::{TimelineId, TimelinePosition, TimelineState};

use super::{PlaybackDirection, RepeatCount};

#[doc = include_str!("../../docs/api/timeline/timeline.doc/Timeline.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Timeline {
    /// The stable identity of this timeline (spec §4.17 "stable identity").
    id: TimelineId,
    /// Logical seconds for one iteration. Always strictly positive: a `0` or
    /// negative duration passed to [`new`](Self::new) is clamped up to
    /// [`MIN_DURATION`](Self::MIN_DURATION) to keep progress division safe.
    duration: f32,
    /// The current lifecycle state.
    state: TimelineState,
    /// The current raw linear position.
    position: TimelinePosition,
    /// The direction the current iteration is playing.
    direction: PlaybackDirection,
    /// How many iterations this timeline plays.
    repeat: RepeatCount,
    /// Whether each iteration flips [`direction`](Self::direction) (ping-pong).
    alternates: bool,
    /// The authored start delay, in logical seconds. Positive delays the first
    /// iteration; negative seeks that far into the playback at
    /// [`start`](Self::start).
    delay: f32,
    /// Logical seconds of the (positive) delay still to elapse before the first
    /// iteration advances. Always `0` once the delay is spent.
    delay_remaining: f32,
}

impl Timeline {
    #[doc = include_str!("../../docs/api/timeline/timeline.doc/MIN_DURATION.md")]
    pub const MIN_DURATION: f32 = f32::EPSILON;

    #[doc = include_str!("../../docs/api/timeline/timeline.doc/new.md")]
    #[must_use]
    pub fn new(id: TimelineId, duration: f32) -> Self {
        Self {
            id,
            duration: if duration.is_finite() {
                duration.max(Self::MIN_DURATION)
            } else {
                Self::MIN_DURATION
            },
            state: TimelineState::Idle,
            position: TimelinePosition::start(),
            direction: PlaybackDirection::Forward,
            repeat: RepeatCount::Finite(1),
            alternates: false,
            delay: 0.0,
            delay_remaining: 0.0,
        }
    }

    #[doc = include_str!("../../docs/api/timeline/timeline.doc/with_delay.md")]
    #[must_use]
    pub fn with_delay(mut self, delay: f32) -> Self {
        self.delay = if delay.is_nan() { 0.0 } else { delay };
        self
    }

    /// Sets the repeat count, returning the updated timeline.
    #[must_use]
    pub fn with_repeat(mut self, repeat: RepeatCount) -> Self {
        self.repeat = repeat;
        self
    }

    /// Sets the playback direction, returning the updated timeline.
    #[must_use]
    pub fn with_direction(mut self, direction: PlaybackDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Sets whether the timeline alternates direction each iteration, returning
    /// the updated timeline.
    #[must_use]
    pub fn alternating(mut self, alternates: bool) -> Self {
        self.alternates = alternates;
        self
    }

    #[doc = include_str!("../../docs/api/timeline/timeline.doc/start.md")]
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

    /// Pauses a running timeline. Has no effect unless the timeline is
    /// [`Running`](TimelineState::Running).
    pub fn pause(&mut self) {
        if self.state.is_running() {
            self.state = TimelineState::Paused;
        }
    }

    /// Resumes a paused timeline. Has no effect unless the timeline is
    /// [`Paused`](TimelineState::Paused).
    pub fn resume(&mut self) {
        if matches!(self.state, TimelineState::Paused) {
            self.state = TimelineState::Running;
        }
    }

    #[doc = include_str!("../../docs/api/timeline/timeline.doc/advance.md")]
    pub fn advance(&mut self, delta: f32) {
        if !self.state.is_running() {
            return;
        }

        // Remaining logical time still to consume this call (always the magnitude;
        // the current `direction` decides whether it grows or shrinks `elapsed`).
        let mut remaining = delta;
        if !remaining.is_finite() || remaining <= 0.0 {
            // A non-finite or non-positive delta does not move the timeline;
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

        let duration = f64::from(self.duration);
        let elapsed = f64::from(self.position.elapsed());
        let phase = match self.direction {
            PlaybackDirection::Forward => elapsed,
            PlaybackDirection::Reverse => duration - elapsed,
        };
        let remaining = f64::from(remaining);
        if remaining < duration - phase {
            let elapsed = match self.direction {
                PlaybackDirection::Forward => elapsed + remaining,
                PlaybackDirection::Reverse => elapsed - remaining,
            };
            #[allow(clippy::cast_possible_truncation)]
            let elapsed = elapsed as f32;
            self.position = TimelinePosition::new(
                elapsed,
                (elapsed / self.duration).clamp(0.0, 1.0),
                self.position.iteration(),
            );
            return;
        }
        // Reduce the delta before adding the current phase so a huge step cannot
        // round away the partial iteration we started in.
        let remainder = remaining % duration;
        let whole = ((remaining - remainder) / duration).round();
        let partial = remainder + phase;
        let carry = partial >= duration;
        let crossed = whole + f64::from(u8::from(carry));
        let iteration = self.position.iteration();

        if let RepeatCount::Finite(count) = self.repeat {
            let left = count.saturating_sub(iteration).max(1);
            if crossed >= f64::from(left) {
                if self.alternates && left % 2 == 0 {
                    self.direction = self.direction.flipped();
                }
                let elapsed = match self.direction {
                    PlaybackDirection::Forward => self.duration,
                    PlaybackDirection::Reverse => 0.0,
                };
                self.position = TimelinePosition::new(
                    elapsed,
                    elapsed / self.duration,
                    iteration.saturating_add(left),
                );
                self.state = TimelineState::Finished;
                return;
            }
        }

        // The modulo over two durations retains parity even when the quotient
        // exceeds the precision of f64 or the iteration counter saturates.
        if self.alternates && (remaining % (duration * 2.0) >= duration) != carry {
            self.direction = self.direction.flipped();
        }
        let phase = if carry { partial - duration } else { partial };
        let elapsed = match self.direction {
            PlaybackDirection::Forward => phase,
            PlaybackDirection::Reverse => duration - phase,
        };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let iterations = crossed as u32;
        #[allow(clippy::cast_possible_truncation)]
        let elapsed = elapsed as f32;
        self.position = TimelinePosition::new(
            elapsed,
            (elapsed / self.duration).clamp(0.0, 1.0),
            iteration.saturating_add(iterations),
        );
    }

    #[doc = include_str!("../../docs/api/timeline/timeline.doc/reverse.md")]
    pub fn reverse(&mut self) {
        self.direction = self.direction.flipped();
        let elapsed = (self.duration - self.position.elapsed()).clamp(0.0, self.duration);
        let progress = (1.0 - self.position.progress()).clamp(0.0, 1.0);
        self.position = TimelinePosition::new(elapsed, progress, self.position.iteration());
    }

    /// Leaves the timeline paused at a normalized progress, skipping any pending start delay.
    pub fn scrub(&mut self, progress: f32) {
        if !progress.is_finite() || !self.state.is_active() {
            return;
        }
        self.pause();
        self.delay_remaining = 0.0;
        self.seek(progress);
    }

    #[doc = include_str!("../../docs/api/timeline/timeline.doc/seek.md")]
    pub fn seek(&mut self, progress: f32) {
        let progress = progress.clamp(0.0, 1.0);
        self.position = TimelinePosition::new(
            progress * self.duration,
            progress,
            self.position.iteration(),
        );
    }

    /// Returns the position at the start of an iteration in the current
    /// direction: progress `0` (elapsed `0`) forward, progress `1` (elapsed
    /// `duration`) reverse.
    fn iteration_start_position(&self) -> TimelinePosition {
        match self.direction {
            PlaybackDirection::Forward => TimelinePosition::new(0.0, 0.0, 0),
            PlaybackDirection::Reverse => TimelinePosition::new(self.duration, 1.0, 0),
        }
    }

    /// Returns the timeline's stable identity.
    #[must_use]
    pub const fn id(&self) -> TimelineId {
        self.id
    }

    /// Returns the iteration duration in logical seconds (always strictly
    /// positive).
    #[must_use]
    pub const fn duration(&self) -> f32 {
        self.duration
    }

    /// Returns the current lifecycle state.
    #[must_use]
    pub const fn state(&self) -> TimelineState {
        self.state
    }

    /// Returns the current raw linear position.
    #[must_use]
    pub const fn position(&self) -> TimelinePosition {
        self.position
    }

    /// Returns the current normalized progress within the iteration
    /// (`position().progress()`).
    #[must_use]
    pub const fn progress(&self) -> f32 {
        self.position.progress()
    }

    /// Returns whether the timeline has finished all its iterations.
    #[must_use]
    pub const fn is_finished(&self) -> bool {
        self.state.is_finished()
    }

    /// Returns the current playback direction.
    #[must_use]
    pub const fn direction(&self) -> PlaybackDirection {
        self.direction
    }

    /// Returns the repeat count.
    #[must_use]
    pub const fn repeat(&self) -> RepeatCount {
        self.repeat
    }

    /// Returns whether the timeline alternates direction each iteration.
    #[must_use]
    pub const fn alternates(&self) -> bool {
        self.alternates
    }

    /// Returns the authored start delay in logical seconds (negative means the
    /// timeline seeks into its playback at [`start`](Self::start)).
    #[must_use]
    pub const fn delay(&self) -> f32 {
        self.delay
    }

    #[doc = include_str!("../../docs/api/timeline/timeline.doc/is_delayed.md")]
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
        assert_eq!(
            Timeline::new(TimelineId::from_raw(1), f32::INFINITY).duration(),
            Timeline::MIN_DURATION,
        );
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

    /// Checks that invalid steps leave finite and infinite repeats running at the same position.
    #[test]
    fn non_finite_steps_leave_a_repeating_timeline_unchanged() {
        for repeat in [RepeatCount::Finite(2), RepeatCount::Infinite] {
            let mut timeline = timeline().with_repeat(repeat);
            timeline.start();
            timeline.advance(0.25);
            let position = timeline.position();
            for delta in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
                timeline.advance(delta);
                assert_eq!(timeline.position(), position);
                assert_eq!(timeline.state(), TimelineState::Running);
            }
            timeline.advance(0.25);
            assert_eq!(timeline.progress(), 0.5);
        }
    }

    /// Exercises iteration skipping and alternating direction with a step large enough to saturate the counter.
    #[test]
    fn large_finite_steps_skip_repeats() {
        let mut infinite = timeline()
            .with_repeat(RepeatCount::Infinite)
            .alternating(true);
        infinite.start();
        infinite.advance(0.25);
        infinite.advance(f32::MAX);
        assert_eq!(infinite.position().iteration(), u32::MAX);
        assert_eq!(infinite.direction(), PlaybackDirection::Forward);
        assert_eq!(infinite.progress(), 0.25);
        infinite.advance(1.0);
        assert_eq!(infinite.direction(), PlaybackDirection::Reverse);
        assert_eq!(infinite.progress(), 0.75);

        let mut odd = Timeline::new(TimelineId::from_raw(2), 11.0)
            .with_repeat(RepeatCount::Infinite)
            .alternating(true);
        odd.start();
        odd.advance(f32::MAX);
        // f32::MAX modulo 22 is 20: one odd iteration and 9 seconds into it.
        assert_eq!(odd.direction(), PlaybackDirection::Reverse);
        assert_eq!(odd.position().elapsed(), 2.0);

        let mut finite = timeline()
            .with_repeat(RepeatCount::Finite(u32::MAX - 1))
            .alternating(true);
        finite.start();
        finite.advance(f32::MAX);
        assert!(finite.is_finished());
        assert_eq!(finite.position().iteration(), u32::MAX - 1);
        assert_eq!(finite.progress(), 0.0);
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
