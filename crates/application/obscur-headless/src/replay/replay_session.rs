//! Deterministic replay of an [`EventRecording`] against test infrastructure (spec §7.13).

use super::{
    EnvironmentChange, EventRecording, RecordedEvent, ReplayResult, ReplayStatus, ReplayTarget,
    SchedulerReason,
};
use crate::host::HeadlessHost;
use crate::input::{SyntheticDispatcher, SyntheticInput};

#[doc = include_str!("../../docs/api/replay/replay_session.doc/ReplaySession.md")]
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

    #[doc = include_str!("../../docs/api/replay/replay_session.doc/replay.md")]
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

    /// Replays into a real headless runtime. Accepted frame decisions render
    /// the supplied view. Stops at the first unresolved event or unsupported
    /// operation and returns its step as a divergence.
    pub fn replay_on_host(
        &self,
        host: &mut HeadlessHost,
        view: &obscur_view::view::View,
        sheets: &[obscur_style::matching::Stylesheet],
    ) -> ReplayResult {
        let mut applied = 0;
        let mut redacted = 0;
        let mut nanos = 0u64;
        let mut completions = 0;
        let mut steps = 0;
        for step in self.recording.steps() {
            match &step.event {
                RecordedEvent::Input(input) => {
                    let outcome = match self.recording.pointer_timestamp(step) {
                        Some(timestamp) => SyntheticDispatcher::new()
                            .at_time(timestamp)
                            .dispatch(host.runtime_mut(), input),
                        None => host.dispatch_input(input),
                    };
                    // A key can reach a window command without a focused node.
                    if !outcome.resolved() && !matches!(input, SyntheticInput::Key(_)) {
                        break;
                    }
                    applied += 1;
                }
                RecordedEvent::ClockTick { nanos: delta } => {
                    host.advance_time(*delta);
                    nanos = nanos.saturating_add(*delta);
                }
                RecordedEvent::AsyncCompletion { operation } => {
                    if !host.complete_operation(*operation) {
                        break;
                    }
                    completions += 1;
                }
                RecordedEvent::SchedulerDecision {
                    accepted: false, ..
                } => {}
                RecordedEvent::SchedulerDecision {
                    reason: SchedulerReason::Frame,
                    accepted: true,
                } => {
                    if host.run_recorded_frame(view, sheets).is_none() {
                        break;
                    }
                }
                RecordedEvent::FrameRequest { reason, decision } => {
                    if host.runtime_mut().request_frame(*reason) != *decision {
                        break;
                    }
                }
                RecordedEvent::SchedulerDecision {
                    reason: SchedulerReason::Timer,
                    ..
                } => {
                    if !host.run_due_timer() {
                        break;
                    }
                }
                RecordedEvent::SchedulerDecision {
                    reason: SchedulerReason::Idle,
                    ..
                } => {
                    if host.runtime_mut().service_idle_work() == 0 {
                        break;
                    }
                }
                RecordedEvent::EnvironmentChange(EnvironmentChange::WindowResized {
                    width,
                    height,
                }) => {
                    host.resize_window(obscur_geometry::Size::new(*width as f32, *height as f32));
                }
                RecordedEvent::EnvironmentChange(EnvironmentChange::VisibilityChanged {
                    visible,
                }) => {
                    host.set_window_visibility(*visible);
                }
                RecordedEvent::Redacted { .. } => redacted += 1,
                RecordedEvent::ResourceVersion { resource, version } => {
                    let id = obscur_identity::ResourceId::from_raw(*resource);
                    if host
                        .runtime()
                        .resource_version(id)
                        .map(|value| value.stamp().raw())
                        != Some(*version)
                    {
                        break;
                    }
                }
                RecordedEvent::SchedulerDecision { .. } => break,
            }
            steps += 1;
        }
        let status = if steps == self.recording.len() {
            ReplayStatus::Completed
        } else {
            ReplayStatus::Diverged { at_step: steps }
        };
        ReplayResult::new(steps, redacted, applied, nanos, completions, status)
    }

    #[doc = include_str!("../../docs/api/replay/replay_session.doc/replay_checked.md")]
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
            RecordedEvent::FrameRequest { reason, decision } => {
                target.frame_request(*reason, *decision);
            }
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

#[cfg(test)]
mod tests {
    use super::super::{CollectingReplayTarget, Sensitivity};
    use super::*;
    use crate::input::{PointerButton, SelectorTarget, SyntheticInput};
    use obscur_identity::NodeId;
    use obscur_runtime::{frame::FrameReason, scheduler::BackpressureDecision};

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
    fn replay_validates_coalescing_of_concrete_frame_requests() {
        use obscur_identity::RuntimeId;
        use obscur_view::view::View;

        for (decision, expected) in [
            (BackpressureDecision::Coalesce, ReplayStatus::Completed),
            (
                BackpressureDecision::Admit,
                ReplayStatus::Diverged { at_step: 1 },
            ),
        ] {
            let mut host = HeadlessHost::new(RuntimeId::from_raw(1));
            let view = View::default();
            let _ = host.runtime_mut().run_due_frame(&view, &[]);
            let mut recording = EventRecording::new();
            recording.record(
                0,
                RecordedEvent::FrameRequest {
                    reason: FrameReason::UiCommit,
                    decision: BackpressureDecision::Admit,
                },
            );
            recording.record(
                1,
                RecordedEvent::FrameRequest {
                    reason: FrameReason::UiCommit,
                    decision,
                },
            );
            recording.record(
                2,
                RecordedEvent::SchedulerDecision {
                    reason: SchedulerReason::Frame,
                    accepted: true,
                },
            );
            let result = ReplaySession::new(recording).replay_on_host(&mut host, &view, &[]);
            assert_eq!(result.status(), expected);
            assert_eq!(
                host.runtime().pending_frames(),
                usize::from(expected != ReplayStatus::Completed)
            );
        }
    }

    #[test]
    fn replay_runs_due_timers_then_services_idle_work_without_rendering() {
        use obscur_identity::RuntimeId;
        use obscur_view::view::View;
        use std::{cell::RefCell, rc::Rc};

        let mut host = HeadlessHost::new(RuntimeId::from_raw(1));
        let view = View::default();
        let _ = host.runtime_mut().run_due_frame(&view, &[]);
        let log = Rc::new(RefCell::new(Vec::new()));
        let timer_log = log.clone();
        host.schedule_timer(10, move |runtime| {
            timer_log.borrow_mut().push("timer");
            runtime.enqueue_background(move || timer_log.borrow_mut().push("idle"));
        });
        let timer = RecordedEvent::SchedulerDecision {
            reason: SchedulerReason::Timer,
            accepted: true,
        };
        let mut early = EventRecording::new();
        early.record(0, timer.clone());
        assert_eq!(
            ReplaySession::new(early)
                .replay_on_host(&mut host, &view, &[])
                .status(),
            ReplayStatus::Diverged { at_step: 0 },
        );
        assert!(log.borrow().is_empty());

        let generation = host.snapshot().generation();
        let mut recording = EventRecording::new();
        recording.record(0, RecordedEvent::ClockTick { nanos: 10 });
        recording.record(1, timer);
        recording.record(
            2,
            RecordedEvent::SchedulerDecision {
                reason: SchedulerReason::Idle,
                accepted: true,
            },
        );
        assert!(
            ReplaySession::new(recording)
                .replay_on_host(&mut host, &view, &[])
                .is_complete()
        );
        assert_eq!(*log.borrow(), ["timer", "idle"]);
        assert_eq!(host.snapshot().generation(), generation);
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
