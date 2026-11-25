use obscur_runtime::{frame::FrameReason, scheduler::BackpressureDecision};

use crate::input::SyntheticInput;

use super::{EnvironmentChange, SchedulerReason};

#[doc = include_str!("../../docs/api/replay/replay_session.doc/ReplayTarget.md")]
pub trait ReplayTarget {
    /// Applies one normalized platform input event.
    fn apply_input(&mut self, input: &SyntheticInput);

    /// Advances the deterministic clock by `nanos` nanoseconds.
    fn advance_clock(&mut self, nanos: u64);

    /// Delivers an async completion for the given raw `operation` id.
    fn complete_async(&mut self, operation: u64);

    /// Observes a recorded scheduler decision. Defaults to a no-op.
    fn scheduler_decision(&mut self, _reason: SchedulerReason, _accepted: bool) {}

    /// Receives a recorded frame request and its expected admission decision.
    fn frame_request(&mut self, _reason: FrameReason, _decision: BackpressureDecision) {}

    /// Observes a recorded resource version change. Defaults to a no-op.
    fn resource_version(&mut self, _resource: u64, _version: u64) {}

    /// Observes a recorded window/environment change. Defaults to a no-op.
    fn environment_change(&mut self, _change: EnvironmentChange) {}
}
