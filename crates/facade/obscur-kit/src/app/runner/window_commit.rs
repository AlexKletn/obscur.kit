//! Bookkeeping for a submitted window frame.

use std::time::{Duration, Instant};

/// Frame timing and size retained until the presentation completes.
pub(super) struct WindowCommit {
    pub(super) extent: (u32, u32),
    pub(super) frame: u64,
    pub(super) present_at: Instant,
    pub(super) acquire: Duration,
    pub(super) built: Duration,
    pub(super) snapshotted: Duration,
    pub(super) work: Duration,
    pub(super) warmup: bool,
    pub(super) prepared_ahead: bool,
}
