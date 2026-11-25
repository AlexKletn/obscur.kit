use std::time::Duration;

/// One measured frame: the CPU (layout/compose) and GPU (present) durations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameSample {
    /// Time spent producing the frame on the CPU (the runtime's reactive frame:
    /// build → reconcile → style → layout → compose).
    pub cpu: Duration,
    /// Time spent presenting the frame on the GPU (submit + blit to surface).
    pub present: Duration,
}

impl FrameSample {
    /// The total wall-clock cost of the frame (CPU + present).
    #[must_use]
    pub fn total(self) -> Duration {
        self.cpu + self.present
    }
}
