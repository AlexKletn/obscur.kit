use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::AcquiredImage;

#[doc = include_str!("../../../docs/api/renderer/surface_acquirer.doc/AcquireState.md")]
#[derive(Default)]
pub(super) struct AcquireState {
    #[doc = include_str!("../../../docs/api/renderer/surface_acquirer.doc/ready.md")]
    pub(super) ready: Option<(AcquiredImage, Duration)>,
    #[doc = include_str!("../../../docs/api/renderer/surface_acquirer.doc/in_flight.md")]
    pub(super) in_flight: bool,
    pub(super) presented_at: Option<Instant>,
    #[doc = include_str!("../../../docs/api/renderer/surface_acquirer.doc/resize_to.md")]
    pub(super) resize_to: Option<(u32, u32)>,
    #[doc = include_str!("../../../docs/api/renderer/surface_acquirer.doc/extent.md")]
    pub(super) extent: Option<(u32, u32)>,
    #[doc = include_str!("../../../docs/api/renderer/surface_acquirer.doc/stopping.md")]
    pub(super) stopping: bool,
}

/// The one lock both the loop and the acquire thread reason about the surface's
/// single image through.
pub(super) type SharedAcquireState = Arc<Mutex<AcquireState>>;

#[doc = include_str!("../../../docs/api/renderer/surface_acquirer.doc/lock.md")]
pub(super) fn lock(shared: &SharedAcquireState) -> std::sync::MutexGuard<'_, AcquireState> {
    shared.lock().unwrap_or_else(|held| held.into_inner())
}
