#![doc = include_str!("replay_session.doc/_module.md")]

use crate::input::SyntheticInput;
use crate::replay::event_recording::{
    EnvironmentChange, EventRecording, RecordedEvent, SchedulerReason,
};
use crate::replay::replay_result::{ReplayResult, ReplayStatus};

#[doc = include_str!("replay_session.doc/ReplayTarget.md")]
pub trait ReplayTarget {
    #[doc = include_str!("replay_session.doc/apply_input.md")]
    fn apply_input(&mut self, input: &SyntheticInput);

    #[doc = include_str!("replay_session.doc/advance_clock.md")]
    fn advance_clock(&mut self, nanos: u64);

    #[doc = include_str!("replay_session.doc/complete_async.md")]
    fn complete_async(&mut self, operation: u64);

    #[doc = include_str!("replay_session.doc/scheduler_decision.md")]
    fn scheduler_decision(&mut self, _reason: SchedulerReason, _accepted: bool) {}

    #[doc = include_str!("replay_session.doc/resource_version.md")]
    fn resource_version(&mut self, _resource: u64, _version: u64) {}

    #[doc = include_str!("replay_session.doc/environment_change.md")]
    fn environment_change(&mut self, _change: EnvironmentChange) {}
}

#[doc = include_str!("replay_session.doc/ReplaySession.md")]
#[derive(Debug, Clone)]
pub struct ReplaySession {
    recording: EventRecording,
}

impl ReplaySession {
    #[doc = include_str!("replay_session.doc/new.md")]
    #[must_use]
    pub fn new(recording: EventRecording) -> Self {
        Self { recording }
    }

    #[doc = include_str!("replay_session.doc/recording.md")]
    #[must_use]
    pub fn recording(&self) -> &EventRecording {
        &self.recording
    }

    #[doc = include_str!("replay_session.doc/replay.md")]
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

    #[doc = include_str!("replay_session.doc/replay_checked.md")]
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

    #[doc = include_str!("replay_session.doc/dispatch.md")]
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

#[doc = include_str!("replay_session.doc/CollectingReplayTarget.md")]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CollectingReplayTarget {
    #[doc = include_str!("replay_session.doc/inputs.md")]
    pub inputs: Vec<SyntheticInput>,
    #[doc = include_str!("replay_session.doc/completions.md")]
    pub completions: Vec<u64>,
    #[doc = include_str!("replay_session.doc/scheduler_decisions.md")]
    pub scheduler_decisions: Vec<(SchedulerReason, bool)>,
    #[doc = include_str!("replay_session.doc/resource_versions.md")]
    pub resource_versions: Vec<(u64, u64)>,
    #[doc = include_str!("replay_session.doc/environment_changes.md")]
    pub environment_changes: Vec<EnvironmentChange>,
    #[doc = include_str!("replay_session.doc/clock_nanos.md")]
    pub clock_nanos: u64,
}

impl CollectingReplayTarget {
    #[doc = include_str!("replay_session.doc/new_2.md")]
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
        SyntheticInput::pointer_down(
            SelectorTarget::node(NodeId::from_raw(raw)),
            PointerButton::Primary,
        )
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
        rec.record(
            1,
            RecordedEvent::ResourceVersion {
                resource: 3,
                version: 9,
            },
        );
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

        assert_eq!(
            target.scheduler_decisions,
            vec![(SchedulerReason::Frame, true)]
        );
        assert_eq!(target.resource_versions, vec![(3, 9)]);
        assert_eq!(
            target.environment_changes,
            vec![EnvironmentChange::WindowResized {
                width: 800,
                height: 600
            }]
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
