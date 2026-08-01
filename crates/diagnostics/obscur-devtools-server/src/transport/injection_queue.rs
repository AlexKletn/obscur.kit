#![doc = include_str!("injection_queue.doc/_module.md")]

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Weak};

use obscur_devtools_protocol::message::{InjectedEvent, MESSAGE_LIMIT};

#[doc = include_str!("injection_queue.doc/InjectionQueue.md")]
#[derive(Debug, Clone, Default)]
pub struct InjectionQueue {
    inner: Arc<Mutex<Inner>>,
}

#[doc = include_str!("injection_queue.doc/Inner.md")]
#[derive(Debug, Default)]
struct Inner {
    #[doc = include_str!("injection_queue.doc/pending.md")]
    pending: VecDeque<InjectedEvent>,
    #[doc = include_str!("injection_queue.doc/dropped.md")]
    dropped: u64,
}

impl InjectionQueue {
    #[doc = include_str!("injection_queue.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("injection_queue.doc/sink.md")]
    #[must_use]
    pub fn sink(&self) -> InjectionSink {
        InjectionSink {
            inner: Arc::downgrade(&self.inner),
        }
    }

    #[doc = include_str!("injection_queue.doc/drain.md")]
    #[must_use]
    pub fn drain(&self) -> Vec<InjectedEvent> {
        let mut inner = self.lock();
        inner.pending.drain(..).collect()
    }

    #[doc = include_str!("injection_queue.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.lock().pending.len()
    }

    #[doc = include_str!("injection_queue.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lock().pending.is_empty()
    }

    #[doc = include_str!("injection_queue.doc/dropped_2.md")]
    #[must_use]
    pub fn dropped(&self) -> u64 {
        self.lock().dropped
    }

    #[doc = include_str!("injection_queue.doc/lock.md")]
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[doc = include_str!("injection_queue.doc/InjectionSink.md")]
#[derive(Debug, Clone, Default)]
pub struct InjectionSink {
    inner: Weak<Mutex<Inner>>,
}

impl InjectionSink {
    #[doc = include_str!("injection_queue.doc/disconnected.md")]
    #[must_use]
    pub fn disconnected() -> Self {
        Self::default()
    }

    #[doc = include_str!("injection_queue.doc/is_live.md")]
    #[must_use]
    pub fn is_live(&self) -> bool {
        self.inner.strong_count() > 0
    }

    #[doc = include_str!("injection_queue.doc/deliver.md")]
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

#[doc = include_str!("injection_queue.doc/PartialEq.md")]
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
