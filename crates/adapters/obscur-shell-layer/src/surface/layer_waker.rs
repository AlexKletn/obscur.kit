//! [`LayerWaker`]: waking the surface's event loop from anywhere.

use std::sync::Arc;

use calloop::ping::{Ping, PingSource, make_ping};

use crate::error::LayerShellError;

#[doc = include_str!("../../docs/api/surface/layer_waker.doc/LayerWaker.md")]
#[derive(Debug, Clone)]
pub struct LayerWaker {
    ping: Ping,
    /// Taken by the loop when the surface is opened. `Arc<Mutex<…>>` rather than
    /// a plain field because a waker is cloned before the loop exists and only
    /// one clone can hand the source over.
    source: Arc<std::sync::Mutex<Option<PingSource>>>,
}

impl LayerWaker {
    #[doc = include_str!("../../docs/api/surface/layer_waker.doc/new.md")]
    pub fn new() -> Result<Self, LayerShellError> {
        let (ping, source) = make_ping().map_err(LayerShellError::WakeSource)?;
        Ok(Self {
            ping,
            source: Arc::new(std::sync::Mutex::new(Some(source))),
        })
    }

    #[doc = include_str!("../../docs/api/surface/layer_waker.doc/wake.md")]
    pub fn wake(&self) {
        self.ping.ping();
    }

    /// Registers the source once, retaining it if registration fails.
    pub(super) fn register_source(
        &self,
        register: impl FnOnce(
            PingSource,
        )
            -> Result<calloop::RegistrationToken, calloop::InsertError<PingSource>>,
    ) -> Result<(), LayerShellError> {
        // Keep the slot locked until registration succeeds or the source is returned.
        let mut slot = self.source.lock().map_err(|_| {
            LayerShellError::Dispatch("layer wake source registration lock was poisoned".into())
        })?;
        if let Some(source) = slot.take()
            && let Err(error) = register(source)
        {
            *slot = Some(error.inserted);
            return Err(LayerShellError::RegisterSource(error.error));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn the_source_is_handed_over_exactly_once() {
        let waker = LayerWaker::new().expect("a ping");
        let clone = waker.clone();
        let event_loop = calloop::EventLoop::<()>::try_new().unwrap();
        clone
            .register_source(|source| event_loop.handle().insert_source(source, |(), (), ()| {}))
            .unwrap();
        waker
            .register_source(|_| panic!("the source was already registered"))
            .unwrap();
    }

    #[test]
    fn a_failed_registration_can_be_retried() {
        let waker = LayerWaker::new().unwrap();
        let failure = waker.register_source(|source| {
            Err(calloop::InsertError {
                inserted: source,
                error: std::io::Error::other("injected registration failure").into(),
            })
        });
        let error = failure.expect_err("registration must report the injected failure");
        assert!(matches!(error, LayerShellError::RegisterSource(_)));
        let cause = error
            .source()
            .and_then(Error::source)
            .and_then(|source| source.downcast_ref::<std::io::Error>());
        assert!(
            matches!(cause, Some(error) if error.to_string() == "injected registration failure")
        );
        let event_loop = calloop::EventLoop::<()>::try_new().unwrap();
        waker
            .register_source(|source| event_loop.handle().insert_source(source, |(), (), ()| {}))
            .unwrap();
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
