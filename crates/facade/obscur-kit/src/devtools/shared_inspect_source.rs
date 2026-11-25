use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use obscur_devtools_protocol::message::{Inspectable, SnapshotPayload, StableSelector};
use obscur_devtools_server::bridge::{InspectSource, SnapshotInspectSource};

#[doc = include_str!("../../docs/api/devtools.doc/SharedInspectSource.md")]
#[derive(Clone, Default)]
pub struct SharedInspectSource {
    inner: Arc<Mutex<SnapshotInspectSource>>,
    /// Live DevTools sessions; the host only publishes while this is non-zero.
    clients: Arc<AtomicUsize>,
}

impl SharedInspectSource {
    /// Creates a shared source backed by an empty snapshot, with no clients.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("../../docs/api/devtools.doc/set.md")]
    pub fn set(&self, source: SnapshotInspectSource) {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = source;
    }

    /// Whether at least one DevTools session is currently connected — the host's
    /// gate for doing the (expensive) live-source build at all.
    #[must_use]
    pub fn has_client(&self) -> bool {
        self.clients.load(Ordering::Relaxed) > 0
    }

    /// Registers a newly accepted DevTools session (called by the serving side).
    pub fn client_connected(&self) {
        self.clients.fetch_add(1, Ordering::Relaxed);
    }

    /// Retires a finished DevTools session (called by the serving side).
    pub fn client_disconnected(&self) {
        // Saturating: an unmatched disconnect must never wrap to "many clients".
        let _ = self
            .clients
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                Some(n.saturating_sub(1))
            });
    }
}

impl InspectSource for SharedInspectSource {
    fn select_stable(&self, selector: &StableSelector) -> SnapshotPayload {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .select_stable(selector)
    }
    fn select_semantic(&self, role: &str, name: Option<&str>) -> SnapshotPayload {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .select_semantic(role, name)
    }
    fn snapshot(&self, target: Inspectable) -> SnapshotPayload {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .snapshot(target)
    }
}
