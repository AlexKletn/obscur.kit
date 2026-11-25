use super::RecordedEvent;

#[doc = include_str!("../../docs/api/replay/event_recording.doc/RecordedStep.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct RecordedStep {
    /// The monotonic tick (sequence index or clock value) at record time.
    pub tick: u64,
    /// What happened at this tick.
    pub event: RecordedEvent,
}

impl RecordedStep {
    /// Creates a recorded step pairing `tick` with `event`.
    #[must_use]
    pub const fn new(tick: u64, event: RecordedEvent) -> Self {
        Self { tick, event }
    }
}
