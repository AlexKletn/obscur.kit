//! Deterministic replay of an [`EventRecording`] against test infrastructure (spec §7.13).

use crate::input::SyntheticInput;
use crate::replay::event_recording::{EnvironmentChange, EventRecording, RecordedEvent, SchedulerReason};
use crate::replay::replay_result::{ReplayResult, ReplayStatus};

/// The sink a [`ReplaySession`] drives while replaying a recording.
///
/// The caller implements this over their deterministic infrastructure
/// (a [`DeterministicClock`](crate::deterministic::DeterministicClock),
/// executor, and completion queue). Replay walks the recorded steps in order and
/// dispatches each to the matching method, so replay runs entirely against
/// deterministic clocks/executors (spec §7.13).
///
/// Only [`apply_input`](Self::apply_input), [`advance_clock`](Self::advance_clock),
/// and [`complete_async`](Self::complete_async) must be implemented. The
/// scheduler-decision, resource-version, and environment-change hooks default to
/// no-ops so a target that does not care about them stays minimal.
pub trait ReplayTarget {
    /// Applies one normalized platform input event.
    fn apply_input(&mut self, input: &SyntheticInput);

    /// Advances the deterministic clock by `nanos` nanoseconds.
    fn advance_clock(&mut self, nanos: u64);

    /// Delivers an async completion for the given raw `operation` id.
    fn complete_async(&mut self, operation: u64);

    /// Observes a recorded scheduler decision. Defaults to a no-op.
    fn scheduler_decision(&mut self, _reason: SchedulerReason, _accepted: bool) {}

    /// Observes a recorded resource version change. Defaults to a no-op.
    fn resource_version(&mut self, _resource: u64, _version: u64) {}

    /// Observes a recorded window/environment change. Defaults to a no-op.
    fn environment_change(&mut self, _change: EnvironmentChange) {}
}

/// A deterministic replayer for an [`EventRecording`].
///
/// [`replay`](Self::replay) walks the recording's fixed step list in order and
/// dispatches each step to a [`ReplayTarget`]. Because the step list is fixed and
/// dispatch is a pure walk, replaying the same recording into equivalent targets
/// always produces the same sequence of calls — the determinism record-replay
/// relies on (spec §7.13). Redaction markers carry no payload and are skipped
/// (but counted).
#[derive(Debug, Clone)]
pub struct ReplaySession {
    recording: EventRecording,
}

impl ReplaySession {
    /// Creates a session that will replay `recording`.
    #[must_use]
    pub fn new(recording: EventRecording) -> Self {
        Self { recording }
    }

    /// Returns the recording this session replays.
    #[must_use]
    pub fn recording(&self) -> &EventRecording {
        &self.recording
    }

    /// Replays every recorded step in order into `target`, returning a tally.
    ///
    /// Each step is dispatched to the matching [`ReplayTarget`] method;
    /// [`Redacted`](RecordedEvent::Redacted) markers are skipped and counted
    /// (their payloads were dropped at record time, spec §7.13). The returned
    /// [`ReplayResult`] always has [`ReplayStatus::Completed`] — a plain replay
    /// of a fixed step list cannot diverge; use
    /// [`replay_checked`](Self::replay_checked) to detect divergence against an
    /// expected recording.
    pub fn replay<T: ReplayTarget>(&self, target: &mut T) -> ReplayResult {
        let mut inputs_applied = 0usize;
        let mut redactions_skipped = 0usize;
        let mut clock_advanced_nanos = 0u64;
        let mut completions = 0usize;

        for step in self.recording.steps() {
            Self::dispatch(
                target,
                &step.event,
                &mut inputs_applied,
                &mut redactions_skipped,
                &mut clock_advanced_nanos,
                &mut completions,
            );
        }

        ReplayResult::new(
            self.recording.len(),
            redactions_skipped,
            inputs_applied,
            clock_advanced_nanos,
            completions,
            ReplayStatus::Completed,
        )
    }

    /// Replays into `target` while checking each step against an `expected`
    /// recording, flagging the first step whose recorded event differs.
    ///
    /// Dispatch is identical to [`replay`](Self::replay); additionally, before
    /// dispatching each step its event is compared to the event at the same
    /// index in `expected`. The first mismatch (including a length mismatch)
    /// sets the status to [`ReplayStatus::Diverged`] with that step index, but
    /// replay still runs to the end so the tallies are complete.
    pub fn replay_checked<T: ReplayTarget>(
        &self,
        target: &mut T,
        expected: &EventRecording,
    ) -> ReplayResult {
        let mut inputs_applied = 0usize;
        let mut redactions_skipped = 0usize;
        let mut clock_advanced_nanos = 0u64;
        let mut completions = 0usize;
        let mut divergence: Option<usize> = None;

        for (index, step) in self.recording.steps().iter().enumerate() {
            let matches_expected = expected
                .steps()
                .get(index)
                .is_some_and(|expected_step| expected_step.event == step.event);
            if !matches_expected && divergence.is_none() {
                divergence = Some(index);
            }

            Self::dispatch(
                target,
                &step.event,
                &mut inputs_applied,
                &mut redactions_skipped,
                &mut clock_advanced_nanos,
                &mut completions,
            );
        }

        // A shorter recording than expected also diverges, at the first missing step.
        if divergence.is_none() && expected.len() > self.recording.len() {
            divergence = Some(self.recording.len());
        }

        let status = match divergence {
            Some(at_step) => ReplayStatus::Diverged { at_step },
            None => ReplayStatus::Completed,
        };

        ReplayResult::new(
            self.recording.len(),
            redactions_skipped,
            inputs_applied,
            clock_advanced_nanos,
            completions,
            status,
        )
    }

    /// Dispatches a single event to `target`, updating the running tallies.
    fn dispatch<T: ReplayTarget>(
        target: &mut T,
        event: &RecordedEvent,
        inputs_applied: &mut usize,
        redactions_skipped: &mut usize,
        clock_advanced_nanos: &mut u64,
        completions: &mut usize,
    ) {
        match event {
            RecordedEvent::Input(input) => {
                target.apply_input(input);
                *inputs_applied += 1;
            }
            RecordedEvent::ClockTick { nanos } => {
                target.advance_clock(*nanos);
                *clock_advanced_nanos = clock_advanced_nanos.saturating_add(*nanos);
            }
            RecordedEvent::AsyncCompletion { operation } => {
                target.complete_async(*operation);
                *completions += 1;
            }
            RecordedEvent::SchedulerDecision { reason, accepted } => {
                target.scheduler_decision(*reason, *accepted);
            }
            RecordedEvent::ResourceVersion { resource, version } => {
                target.resource_version(*resource, *version);
            }
            RecordedEvent::EnvironmentChange(change) => {
                target.environment_change(*change);
            }
            RecordedEvent::Redacted { .. } => {
                // Payload was dropped at record time; nothing to dispatch.
                *redactions_skipped += 1;
            }
        }
    }
}

/// A [`ReplayTarget`] test double that records every dispatched effect, so tests
/// can assert on the exact sequence a replay produced.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CollectingReplayTarget {
    /// Inputs applied, in dispatch order.
    pub inputs: Vec<SyntheticInput>,
    /// Async completion operation ids, in dispatch order.
    pub completions: Vec<u64>,
    /// Scheduler decisions observed, in dispatch order.
    pub scheduler_decisions: Vec<(SchedulerReason, bool)>,
    /// Resource version changes observed, in dispatch order.
    pub resource_versions: Vec<(u64, u64)>,
    /// Environment changes observed, in dispatch order.
    pub environment_changes: Vec<EnvironmentChange>,
    /// The running total the clock was advanced by.
    pub clock_nanos: u64,
}

impl CollectingReplayTarget {
    /// Creates an empty collecting target.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl ReplayTarget for CollectingReplayTarget {
    fn apply_input(&mut self, input: &SyntheticInput) {
        self.inputs.push(input.clone());
    }

    fn advance_clock(&mut self, nanos: u64) {
        self.clock_nanos = self.clock_nanos.saturating_add(nanos);
    }

    fn complete_async(&mut self, operation: u64) {
        self.completions.push(operation);
    }

    fn scheduler_decision(&mut self, reason: SchedulerReason, accepted: bool) {
        self.scheduler_decisions.push((reason, accepted));
    }

    fn resource_version(&mut self, resource: u64, version: u64) {
        self.resource_versions.push((resource, version));
    }

    fn environment_change(&mut self, change: EnvironmentChange) {
        self.environment_changes.push(change);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{PointerButton, SelectorTarget, SyntheticInput};
    use crate::replay::event_recording::Sensitivity;
    use obscur_identity::NodeId;

    fn input(raw: u64) -> SyntheticInput {
        SyntheticInput::pointer_down(SelectorTarget::node(NodeId::from_raw(raw)), PointerButton::Primary)
    }

    fn sample_recording() -> EventRecording {
        let mut rec = EventRecording::new();
        rec.record_input(0, input(1), Sensitivity::Normal);
        rec.record(1, RecordedEvent::ClockTick { nanos: 10 });
        rec.record(2, RecordedEvent::AsyncCompletion { operation: 7 });
        rec.record_input(3, input(2), Sensitivity::Normal);
        rec.record(4, RecordedEvent::ClockTick { nanos: 5 });
        rec
    }

    #[test]
    fn replay_dispatches_in_order_with_matching_counts() {
        let rec = sample_recording();
        let session = ReplaySession::new(rec);
        let mut target = CollectingReplayTarget::new();

        let result = session.replay(&mut target);

        assert_eq!(target.inputs, vec![input(1), input(2)]);
        assert_eq!(target.completions, vec![7]);
        assert_eq!(target.clock_nanos, 15);

        assert_eq!(result.steps_replayed(), 5);
        assert_eq!(result.inputs_applied(), 2);
        assert_eq!(result.completions(), 1);
        assert_eq!(result.clock_advanced_nanos(), 15);
        assert_eq!(result.redactions_skipped(), 0);
        assert!(result.is_complete());
    }

    #[test]
    fn replay_is_deterministic_across_runs() {
        let session = ReplaySession::new(sample_recording());

        let mut first = CollectingReplayTarget::new();
        let result_a = session.replay(&mut first);

        let mut second = CollectingReplayTarget::new();
        let result_b = session.replay(&mut second);

        assert_eq!(first, second);
        assert_eq!(result_a, result_b);
    }

    #[test]
    fn redacted_step_is_counted_and_never_reaches_target() {
        let mut rec = EventRecording::new();
        rec.record_input(0, input(1), Sensitivity::Normal);
        // Redacted by the default policy: payload dropped at record time.
        rec.record_input(1, input(99), Sensitivity::Secret);
        rec.record(2, RecordedEvent::ClockTick { nanos: 4 });

        let session = ReplaySession::new(rec);
        let mut target = CollectingReplayTarget::new();
        let result = session.replay(&mut target);

        assert_eq!(result.steps_replayed(), 3);
        assert_eq!(result.redactions_skipped(), 1);
        assert_eq!(result.inputs_applied(), 1);
        // The secret input never reached the target.
        assert_eq!(target.inputs, vec![input(1)]);
        assert_eq!(target.clock_nanos, 4);
        assert!(result.is_complete());
    }

    #[test]
    fn structured_events_reach_default_hooks() {
        let mut rec = EventRecording::new();
        rec.record(
            0,
            RecordedEvent::SchedulerDecision {
                reason: SchedulerReason::Frame,
                accepted: true,
            },
        );
        rec.record(1, RecordedEvent::ResourceVersion { resource: 3, version: 9 });
        rec.record(
            2,
            RecordedEvent::EnvironmentChange(EnvironmentChange::WindowResized {
                width: 800,
                height: 600,
            }),
        );

        let session = ReplaySession::new(rec);
        let mut target = CollectingReplayTarget::new();
        session.replay(&mut target);

        assert_eq!(target.scheduler_decisions, vec![(SchedulerReason::Frame, true)]);
        assert_eq!(target.resource_versions, vec![(3, 9)]);
        assert_eq!(
            target.environment_changes,
            vec![EnvironmentChange::WindowResized { width: 800, height: 600 }]
        );
    }

    #[test]
    fn replay_checked_completes_on_identical_recording() {
        let session = ReplaySession::new(sample_recording());
        let expected = sample_recording();
        let mut target = CollectingReplayTarget::new();

        let result = session.replay_checked(&mut target, &expected);
        assert!(result.is_complete());
        assert_eq!(result.status(), ReplayStatus::Completed);
    }

    #[test]
    fn replay_checked_flags_first_diverging_step() {
        let session = ReplaySession::new(sample_recording());

        let mut expected = EventRecording::new();
        expected.record_input(0, input(1), Sensitivity::Normal);
        // Differs at step 1: a different clock tick.
        expected.record(1, RecordedEvent::ClockTick { nanos: 999 });
        expected.record(2, RecordedEvent::AsyncCompletion { operation: 7 });
        expected.record_input(3, input(2), Sensitivity::Normal);
        expected.record(4, RecordedEvent::ClockTick { nanos: 5 });

        let mut target = CollectingReplayTarget::new();
        let result = session.replay_checked(&mut target, &expected);

        assert_eq!(result.status(), ReplayStatus::Diverged { at_step: 1 });
        assert!(!result.is_complete());
        // Dispatch still ran fully.
        assert_eq!(result.steps_replayed(), 5);
    }
}
