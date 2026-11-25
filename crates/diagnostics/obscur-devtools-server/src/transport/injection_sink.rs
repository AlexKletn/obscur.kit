use std::sync::{Mutex, Weak};

use obscur_devtools_protocol::message::{InjectedEvent, MESSAGE_LIMIT};

use super::injection_queue_state::InjectionQueueState;

#[doc = include_str!("../../docs/api/transport/injection_queue.doc/InjectionSink.md")]
#[derive(Debug, Clone, Default)]
pub struct InjectionSink {
    inner: Weak<Mutex<InjectionQueueState>>,
}

impl InjectionSink {
    pub(super) fn new(inner: Weak<Mutex<InjectionQueueState>>) -> Self {
        Self { inner }
    }

    #[doc = include_str!("../../docs/api/transport/injection_queue.doc/disconnected.md")]
    #[must_use]
    pub fn disconnected() -> Self {
        Self::default()
    }

    /// Whether the host's queue is still alive, and an event pushed here would
    /// therefore reach something.
    #[must_use]
    pub fn is_live(&self) -> bool {
        self.inner.strong_count() > 0
    }

    #[doc = include_str!("../../docs/api/transport/injection_queue.doc/deliver.md")]
    pub fn deliver(&self, event: InjectedEvent) -> bool {
        let Some(inner) = self.inner.upgrade() else {
            return false;
        };
        let mut guard = inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if guard.pending.len() >= MESSAGE_LIMIT {
            guard.pending.pop_front();
            guard.dropped += 1;
        }
        guard.pending.push_back(event);
        true
    }
}

/// Two sinks are equal when they push into the same queue, so a
/// [`DevToolsEndpoint`](crate::endpoint::DevToolsEndpoint) keeps its derived
/// equality. A `Weak` has no value to compare; identity is the only thing a
/// channel end *has*.
impl PartialEq for InjectionSink {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for InjectionSink {}
