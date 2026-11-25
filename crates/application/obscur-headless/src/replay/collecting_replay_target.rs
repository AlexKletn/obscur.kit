use obscur_runtime::{frame::FrameReason, scheduler::BackpressureDecision};

use crate::input::SyntheticInput;

use super::{EnvironmentChange, ReplayTarget, SchedulerReason};

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
    /// Concrete frame requests and recorded decisions.
    pub frame_requests: Vec<(FrameReason, BackpressureDecision)>,
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
    fn frame_request(&mut self, reason: FrameReason, decision: BackpressureDecision) {
        self.frame_requests.push((reason, decision));
    }

    fn resource_version(&mut self, resource: u64, version: u64) {
        self.resource_versions.push((resource, version));
    }

    fn environment_change(&mut self, change: EnvironmentChange) {
        self.environment_changes.push(change);
    }
}
