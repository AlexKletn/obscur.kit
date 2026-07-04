//! The [`Timeline`] deterministic playable timeline (spec §4.17 "motion engine").

use crate::timeline::{TimelineId, TimelinePosition, TimelineState};

/// The direction a [`Timeline`] plays its iterations (spec §4.17
/// "reversal / alternate").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaybackDirection {
    /// Plays from the start of the iteration (progress `0`) toward its end
    /// (progress `1`).
    Forward,
    /// Plays from the end of the iteration (progress `1`) back toward its start
    /// (progress `0`).
    Reverse,
}

impl PlaybackDirection {
    /// Returns the opposite direction (used when a timeline alternates).
    #[must_use]
    pub const fn flipped(self) -> Self {
        match self {
            Self::Forward => Self::Reverse,
            Self::Reverse => Self::Forward,
        }
    }
}

/// How many times a [`Timeline`] repeats its iteration (spec §4.17 "chaining /
/// repeat").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RepeatCount {
    /// A fixed number of iterations. `Finite(1)` plays exactly once; `Finite(0)`
    /// requests no plays at all and a started timeline is immediately
    /// [`Finished`](TimelineState::Finished).
    Finite(u32),
    /// Repeats forever: the timeline never reaches
    /// [`Finished`](TimelineState::Finished) on its own.
    Infinite,
}

/// A deterministic, time-agnostic playable timeline (spec §4.17 "motion
/// engine").
///
/// A `Timeline` integrates an explicit `delta` of elapsed *logical seconds*
/// (`f32`) supplied by a driver; it never reads a clock. This keeps it
/// deterministic — feeding two identical timelines the same sequence of deltas
/// yields identical positions — and robust under event-driven scheduling and
/// **skipped frames**: a skipped frame is simply a larger `delta` and is handled
/// by the same integration math, with no per-frame assumption baked in.
///
/// The timeline owns its lifecycle ([`TimelineState`]), its raw linear
/// [`TimelinePosition`], a [`PlaybackDirection`], a [`RepeatCount`], and whether
/// it `alternates` direction each iteration. It exposes the primitives the spec
/// requires — start, pause/resume, [`advance`](Self::advance),
/// [`reverse`](Self::reverse) and [`seek`](Self::seek) — from which
/// interruption, reversal, chaining and synchronization are built downstream.
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
}

impl Timeline {
    /// The smallest permitted iteration duration, in logical seconds.
    ///
    /// A non-positive duration would make the `progress = elapsed / duration`
    /// division ill-defined, so [`new`](Self::new) clamps up to this tiny
    /// positive epsilon instead.
    pub const MIN_DURATION: f32 = f32::EPSILON;

    /// Creates an idle timeline of the given identity and iteration `duration`
    /// (logical seconds).
    ///
    /// Defaults: state [`Idle`](TimelineState::Idle), position at
    /// [`start`](TimelinePosition::start), [`Forward`](PlaybackDirection::Forward)
    /// direction, `Finite(1)` (plays once), non-alternating. A `duration` that is
    /// zero, negative or `NaN` is clamped up to
    /// [`MIN_DURATION`](Self::MIN_DURATION).
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
        }
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

    /// Starts the timeline: sets it [`Running`](TimelineState::Running) and
    /// resets the position to the start of the first iteration.
    ///
    /// The reset respects the current direction: a
    /// [`Forward`](PlaybackDirection::Forward) timeline begins at progress `0`
    /// (elapsed `0`), while a [`Reverse`](PlaybackDirection::Reverse) one begins
    /// at progress `1` (elapsed `duration`, its end).
    ///
    /// A timeline whose repeat is `Finite(0)` plays no iterations and is set
    /// straight to [`Finished`](TimelineState::Finished).
    pub fn start(&mut self) {
        self.position = self.iteration_start_position();
        self.state = if matches!(self.repeat, RepeatCount::Finite(0)) {
            TimelineState::Finished
        } else {
            TimelineState::Running
        };
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

    /// Advances the timeline by `delta` logical seconds — the deterministic
    /// integrator (spec §4.17).
    ///
    /// Only a [`Running`](TimelineState::Running) timeline advances; otherwise
    /// this is a no-op. `delta` is added to `elapsed` in the current direction
    /// (forward increases it, reverse decreases it) and progress is recomputed as
    /// `(elapsed / duration).clamp(0, 1)`.
    ///
    /// **Multi-iteration carry.** A single large `delta` — for example one
    /// spanning several skipped frames — may complete more than one iteration. To
    /// handle that, the method consumes the delta in a `while` loop: each pass
    /// fills the time remaining in the current iteration, and any *overshoot* is
    /// carried into the next iteration rather than discarded. When an iteration
    /// completes the [`iteration`](TimelinePosition::iteration) counter increments
    /// (saturating) and, if [`alternates`](Self::alternating) is set, the
    /// direction flips. The loop stops when the delta is exhausted or the repeat
    /// count runs out; in the latter case the position is clamped to the end and
    /// the state becomes [`Finished`](TimelineState::Finished). An
    /// [`Infinite`](RepeatCount::Infinite) timeline never finishes.
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

        loop {
            // Time left before the current iteration completes, in this direction.
            let time_to_end = match self.direction {
                PlaybackDirection::Forward => self.duration - self.position.elapsed(),
                PlaybackDirection::Reverse => self.position.elapsed(),
            };

            if remaining < time_to_end {
                // Stays within the current iteration.
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

            // The current iteration completes; consume the time it took.
            remaining -= time_to_end;
            let completed = self.position.iteration().saturating_add(1);

            let exhausted = match self.repeat {
                RepeatCount::Infinite => false,
                RepeatCount::Finite(count) => completed >= count,
            };

            if exhausted {
                // No iterations remain: clamp to the end of this direction and
                // finish. Forward ends at progress 1, reverse ends at progress 0.
                let (elapsed, progress) = match self.direction {
                    PlaybackDirection::Forward => (self.duration, 1.0),
                    PlaybackDirection::Reverse => (0.0, 0.0),
                };
                self.position = TimelinePosition::new(elapsed, progress, completed);
                self.state = TimelineState::Finished;
                return;
            }

            // Begin the next iteration, flipping direction first if alternating.
            if self.alternates {
                self.direction = self.direction.flipped();
            }
            let (elapsed, progress) = match self.direction {
                PlaybackDirection::Forward => (0.0, 0.0),
                PlaybackDirection::Reverse => (self.duration, 1.0),
            };
            self.position = TimelinePosition::new(elapsed, progress, completed);
            // Loop again to carry `remaining` into this fresh iteration.
        }
    }

    /// Reverses playback, mirroring the current position so a running reversal is
    /// seamless (spec §4.17 "reversal / interruption").
    ///
    /// Flips the direction and mirrors the position within the current iteration:
    /// `elapsed → duration - elapsed` and `progress → 1 - progress`. The position
    /// on screen does not jump; only the direction of travel inverts.
    pub fn reverse(&mut self) {
        self.direction = self.direction.flipped();
        let elapsed = (self.duration - self.position.elapsed()).clamp(0.0, self.duration);
        let progress = (1.0 - self.position.progress()).clamp(0.0, 1.0);
        self.position = TimelinePosition::new(elapsed, progress, self.position.iteration());
    }

    /// Seeks to a normalized `progress` within the current iteration.
    ///
    /// `progress` is clamped to `0..=1` and `elapsed` is set to match; the
    /// iteration counter and state are left unchanged.
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
}
