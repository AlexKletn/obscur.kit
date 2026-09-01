#![doc = include_str!("loop_waker.doc/_module.md")]

#[doc = include_str!("loop_waker.doc/LoopWaker.md")]
pub trait LoopWaker: Send + Sync {
    #[doc = include_str!("loop_waker.doc/wake.md")]
    fn wake(&self);
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[derive(Default)]
    struct Counter(AtomicUsize);

    impl LoopWaker for Counter {
        fn wake(&self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn the_port_is_object_safe() {
        fn assert_obj_safe(_: &dyn LoopWaker) {}
        assert_obj_safe(&Counter::default());
    }

    #[test]
    fn a_waker_crosses_a_thread_boundary() {
        // The whole point of the `Send + Sync` bound: the thread that has news
        // is never the thread that owns the loop.
        let waker: Arc<dyn LoopWaker> = Arc::new(Counter::default());
        let ticker = Arc::clone(&waker);
        std::thread::spawn(move || ticker.wake()).join().unwrap();
        waker.wake();
    }

    #[test]
    fn waking_carries_no_reason() {
        let counter = Counter::default();
        counter.wake();
        counter.wake();
        // Two wakes are two wakes, and neither says why. A caller that needs a
        // reason has to carry it itself, which is the honest arrangement.
        assert_eq!(counter.0.load(Ordering::Relaxed), 2);
    }
}
