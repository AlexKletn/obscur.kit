//! [`LayerWaker`]: waking the surface's event loop from anywhere.

use std::sync::Arc;

use calloop::ping::{Ping, PingSource, make_ping};

use crate::error::LayerShellError;

/// A handle that wakes a layer surface's event loop, from any thread.
///
/// # Why a loop needs waking at all
///
/// The loop sleeps on the compositor's socket. That is the right thing to do —
/// an idle panel should cost nothing — but it means **nothing outside the
/// compositor can make the loop turn**. A worker thread that finished, a timer
/// that fired, a second Wayland connection belonging to some other protocol: all
/// of them are invisible to a loop parked on one file descriptor.
///
/// That last case is not hypothetical. An on-screen keyboard's text-input
/// activation arrives on its *own* connection, on its own descriptor, and the
/// surface's loop polls only its own. Without a waker the keyboard would learn
/// it should appear and have no way to make the frame that shows it.
///
/// # Using it
///
/// Build one before the surface, hand clones wherever they are needed, and give
/// one to the surface at open time. [`wake`](Self::wake) is cheap, thread-safe,
/// and coalesces: several wakes before the loop turns cause one turn, which is
/// what a caller wants — the loop rebuilds from current state, not once per
/// notification.
#[derive(Debug, Clone)]
pub struct LayerWaker {
    ping: Ping,
    /// Taken by the loop when the surface is opened. `Arc<Mutex<…>>` rather than
    /// a plain field because a waker is cloned before the loop exists and only
    /// one clone can hand the source over.
    source: Arc<std::sync::Mutex<Option<PingSource>>>,
}

impl LayerWaker {
    /// Creates a waker and the source its loop will consume.
    ///
    /// # Errors
    ///
    /// [`LayerShellError::Dispatch`] when the operating system refuses the
    /// underlying notification primitive.
    pub fn new() -> Result<Self, LayerShellError> {
        let (ping, source) =
            make_ping().map_err(|error| LayerShellError::Dispatch(error.to_string()))?;
        Ok(Self {
            ping,
            source: Arc::new(std::sync::Mutex::new(Some(source))),
        })
    }

    /// Wakes the loop this waker was given to.
    ///
    /// Does nothing if the surface was never opened, or has closed — a
    /// notification with nowhere to go is not an error.
    pub fn wake(&self) {
        self.ping.ping();
    }

    /// Takes the loop source, leaving none behind.
    pub(super) fn take_source(&self) -> Option<PingSource> {
        self.source.lock().ok().and_then(|mut slot| slot.take())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_source_is_handed_over_exactly_once() {
        let waker = LayerWaker::new().expect("a ping");
        let clone = waker.clone();
        assert!(clone.take_source().is_some(), "the first taker gets it");
        assert!(
            waker.take_source().is_none(),
            "and no clone can hand it over twice"
        );
    }

    /// Waking a loop that does not exist is a no-op, not a panic: a worker
    /// thread should not have to know whether the surface is still open.
    #[test]
    fn waking_with_no_loop_attached_is_harmless() {
        let waker = LayerWaker::new().expect("a ping");
        waker.wake();
        waker.wake();
    }
}
