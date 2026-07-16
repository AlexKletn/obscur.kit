//! `InjectionQueue`/`InjectionSink`: what a host hands the endpoint so an
//! authorized `Inject` reaches a runtime instead of a field nobody can read.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Weak};

use obscur_devtools_protocol::message::{InjectedEvent, MESSAGE_LIMIT};

/// The **host** end of the injection channel: a bounded, thread-shared queue of
/// synthetic input events an authorized DevTools client asked for (spec §7.6
/// agent loop, §7.7 host-provides-device).
///
/// # Why this exists at all
///
/// The endpoint used to hold the injected events in a private `Vec` and hand them
/// out through a `drain_injections(&mut self)`. Nothing ever called it, and the
/// reason is structural rather than an oversight: a host serves its endpoint on
/// its **own thread** (the endpoint owns the session — token, scopes, audit log —
/// and a `UiRuntime` is not `Send`), so the thread that owns the runtime does not
/// own the endpoint and could not reach the drain. Every `Inject` was answered
/// `Acknowledged` and dropped.
///
/// So the channel is inverted, exactly as the *outbound* half already is: the
/// host makes the queue, keeps it, and hands the endpoint a [`sink`](Self::sink).
/// The serving thread pushes; the host drains on its own thread and applies each
/// event to its runtime (`obscur_kit::devtools::apply_injected`). This is the
/// mirror image of [`EventQueue`](crate::transport::EventQueue), and for the same
/// reason: the endpoint decides *whether* it happens, the host decides *what it
/// does*, and neither reaches into the other's thread.
///
/// # What a sink proves, and what it does not
///
/// The endpoint refuses an `Inject` when it holds no live sink, so a host that
/// cannot receive injected input says so instead of acknowledging it — that is
/// the §7-C half this closes, and it is a fact about reachability, not a promise:
/// a [`sink`](Self::sink) holds a [`Weak`], so when the host drops its queue the
/// endpoint's sink goes dead and injection is refused again.
///
/// It does **not** prove the host drains. A host that builds a queue, hands out a
/// sink, keeps it alive and never calls [`drain`](Self::drain) has re-created
/// drop-on-the-floor for itself, and no type here can see that. What is now
/// impossible is the state every host in this workspace was actually in: no
/// receiver at all, and an acknowledgement anyway.
///
/// # Bounded on purpose
///
/// [`MESSAGE_LIMIT`], the same ceiling §7.7 imposes elsewhere. A client that
/// outruns the host's frame rate **drops the oldest** event rather than growing
/// without bound, and [`dropped`](Self::dropped) counts the loss. Unbounded, a
/// paused host plus a chatty agent is a memory leak in the app being inspected.
#[derive(Debug, Clone, Default)]
pub struct InjectionQueue {
    inner: Arc<Mutex<Inner>>,
}

/// The shared state behind an [`InjectionQueue`].
#[derive(Debug, Default)]
struct Inner {
    /// Pending injected events, oldest first.
    pending: VecDeque<InjectedEvent>,
    /// How many events were dropped because the queue was full.
    dropped: u64,
}

impl InjectionQueue {
    /// Creates an empty queue.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the push end to hand to a
    /// [`DevToolsEndpoint`](crate::endpoint::DevToolsEndpoint).
    ///
    /// The sink holds a [`Weak`] reference, so it keeps nothing alive: it is
    /// usable exactly as long as *this* queue — the drain end — is.
    #[must_use]
    pub fn sink(&self) -> InjectionSink {
        InjectionSink {
            inner: Arc::downgrade(&self.inner),
        }
    }

    /// Takes every pending event, leaving the queue empty.
    ///
    /// The host calls this from the thread that owns its runtime (typically once
    /// per frame) and applies each event.
    #[must_use]
    pub fn drain(&self) -> Vec<InjectedEvent> {
        let mut inner = self.lock();
        inner.pending.drain(..).collect()
    }

    /// How many events are waiting.
    #[must_use]
    pub fn len(&self) -> usize {
        self.lock().pending.len()
    }

    /// Whether nothing is waiting.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lock().pending.is_empty()
    }

    /// How many events have been dropped for backpressure since creation.
    #[must_use]
    pub fn dropped(&self) -> u64 {
        self.lock().dropped
    }

    /// Locks the shared state, recovering from a poisoned lock rather than
    /// propagating it: a panicked serving thread must not take the host down with
    /// it — see [`EventQueue`](crate::transport::EventQueue)'s identical note.
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// The **endpoint** end of the injection channel: where an authorized `Inject`
/// is delivered (spec §7.6).
///
/// Obtained from [`InjectionQueue::sink`] and installed with
/// [`DevToolsEndpoint::with_injection_sink`](crate::endpoint::DevToolsEndpoint::with_injection_sink).
/// It holds a [`Weak`], so it reports [`is_live`](Self::is_live) only while the
/// host's queue still exists — an endpoint outliving its host's drain end refuses
/// injection rather than filling a queue nobody will ever read.
#[derive(Debug, Clone, Default)]
pub struct InjectionSink {
    inner: Weak<Mutex<Inner>>,
}

impl InjectionSink {
    /// A sink connected to nothing, which accepts no event.
    ///
    /// This is what a `Default` endpoint effectively has, and it is deliberately
    /// constructible: it is the honest description of a host with no runtime to
    /// inject into (the `obscur-devtools serve` demo host is exactly that).
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

    /// Delivers `event` to the host's queue.
    ///
    /// Returns `false` when there is nothing to deliver to — the host's
    /// [`InjectionQueue`] has been dropped — and the event is discarded. The
    /// endpoint turns that `false` into a refusal rather than an acknowledgement.
    ///
    /// A `true` return with a full queue still means delivered: the oldest event
    /// is evicted and counted in [`dropped`](InjectionQueue::dropped). Refusing
    /// the command would be wrong there — the channel is working, the host is
    /// behind — and the count is where that is reported.
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

#[cfg(test)]
mod tests {
    use super::{InjectionQueue, InjectionSink};
    use obscur_devtools_protocol::message::{InjectedEvent, MESSAGE_LIMIT};

    fn click(x: i32) -> InjectedEvent {
        InjectedEvent::PointerDown { x, y: 0 }
    }

    #[test]
    fn delivered_events_arrive_at_the_host_in_order() {
        let queue = InjectionQueue::new();
        let sink = queue.sink();
        assert!(queue.is_empty());
        assert!(sink.deliver(click(1)));
        assert!(sink.deliver(click(2)));
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.drain(), vec![click(1), click(2)]);
        assert!(queue.is_empty(), "a drain empties the queue");
    }

    #[test]
    fn a_disconnected_sink_delivers_nothing() {
        let sink = InjectionSink::disconnected();
        assert!(!sink.is_live());
        assert!(!sink.deliver(click(1)));
    }

    #[test]
    fn dropping_the_host_end_kills_the_sink() {
        // The point of the `Weak`: an endpoint cannot go on accepting input for a
        // host that no longer exists to receive it.
        let queue = InjectionQueue::new();
        let sink = queue.sink();
        assert!(sink.is_live());
        drop(queue);
        assert!(!sink.is_live());
        assert!(!sink.deliver(click(1)));
    }

    #[test]
    fn a_clone_of_the_host_end_keeps_the_sink_live() {
        // The queue is `Clone` (shared), so holding any handle is holding the
        // drain end — dropping one of two must not disconnect anything.
        let queue = InjectionQueue::new();
        let sink = queue.sink();
        let second = queue.clone();
        drop(queue);
        assert!(sink.is_live());
        assert!(sink.deliver(click(7)));
        assert_eq!(second.drain(), vec![click(7)]);
    }

    #[test]
    fn a_full_queue_drops_the_oldest_and_counts_it() {
        let queue = InjectionQueue::new();
        let sink = queue.sink();
        for x in 0..i32::try_from(MESSAGE_LIMIT).expect("the message limit fits an i32") {
            assert!(sink.deliver(click(x)));
        }
        assert_eq!(queue.dropped(), 0);
        // One past the limit: still delivered, oldest evicted.
        assert!(sink.deliver(click(-1)));
        assert_eq!(queue.dropped(), 1);
        assert_eq!(queue.len(), MESSAGE_LIMIT);
        let drained = queue.drain();
        assert_eq!(drained[0], click(1), "the oldest was the one dropped");
        assert_eq!(drained[MESSAGE_LIMIT - 1], click(-1));
    }

    #[test]
    fn sinks_onto_the_same_queue_are_equal() {
        let queue = InjectionQueue::new();
        assert_eq!(queue.sink(), queue.sink());
        assert_ne!(queue.sink(), InjectionQueue::new().sink());
        assert_eq!(InjectionSink::disconnected(), InjectionSink::disconnected());
    }
}
