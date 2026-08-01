#![doc = include_str!("layer_waker.doc/_module.md")]

use std::sync::Arc;

use calloop::ping::{Ping, PingSource, make_ping};

use crate::error::LayerShellError;

#[doc = include_str!("layer_waker.doc/LayerWaker.md")]
#[derive(Debug, Clone)]
pub struct LayerWaker {
    ping: Ping,
    #[doc = include_str!("layer_waker.doc/source.md")]
    source: Arc<std::sync::Mutex<Option<PingSource>>>,
}

impl LayerWaker {
    #[doc = include_str!("layer_waker.doc/new.md")]
    pub fn new() -> Result<Self, LayerShellError> {
        let (ping, source) =
            make_ping().map_err(|error| LayerShellError::Dispatch(error.to_string()))?;
        Ok(Self {
            ping,
            source: Arc::new(std::sync::Mutex::new(Some(source))),
        })
    }

    #[doc = include_str!("layer_waker.doc/wake.md")]
    pub fn wake(&self) {
        self.ping.ping();
    }

    #[doc = include_str!("layer_waker.doc/take_source.md")]
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
