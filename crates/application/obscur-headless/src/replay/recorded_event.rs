use obscur_runtime::{frame::FrameReason, scheduler::BackpressureDecision};

use crate::input::SyntheticInput;

use super::{EnvironmentChange, SchedulerReason, Sensitivity};

/// One recorded happening, covering the categories a record-replay run may
/// capture (spec §7.13): normalized platform events, scheduler decisions, clock
/// ticks, async completions, resource versions, and window/environment changes —
/// plus a [`Redacted`](RecordedEvent::Redacted) placeholder for input whose
/// payload was dropped by the [`RedactionPolicy`](super::RedactionPolicy).
#[derive(Debug, Clone, PartialEq)]
pub enum RecordedEvent {
    /// A concrete request and the scheduler decision recorded for it.
    FrameRequest {
        /// The work requested from the runtime.
        reason: FrameReason,
        /// The scheduler's expected admission decision.
        decision: BackpressureDecision,
    },
    /// A normalized platform input event.
    Input(SyntheticInput),
    /// A deterministic clock tick, advancing virtual time by `nanos`.
    ClockTick {
        /// Nanoseconds the clock advanced.
        nanos: u64,
    },
    /// An async completion delivered for the given `operation`.
    AsyncCompletion {
        /// The raw operation id the completion resolves.
        operation: u64,
    },
    /// A scheduler decision, recorded as a typed marker.
    SchedulerDecision {
        /// Why the scheduler made this decision.
        reason: SchedulerReason,
        /// Whether the scheduled work was accepted (vs. coalesced/dropped).
        accepted: bool,
    },
    /// A resource whose version changed.
    ResourceVersion {
        /// The raw resource id.
        resource: u64,
        /// The new version of that resource.
        version: u64,
    },
    /// A window/environment change.
    EnvironmentChange(EnvironmentChange),
    /// A placeholder for input whose payload was redacted and dropped. Carries
    /// only the [`Sensitivity`] class, never the original data (spec §7.13).
    Redacted {
        /// Which sensitivity class was redacted.
        kind: Sensitivity,
    },
}
