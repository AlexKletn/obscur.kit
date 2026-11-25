use std::sync::atomic::AtomicBool;

use event_listener::Event;

#[derive(Default)]
pub(super) struct Revocation {
    pub(super) revoked: AtomicBool,
    pub(super) changed: Event,
}
