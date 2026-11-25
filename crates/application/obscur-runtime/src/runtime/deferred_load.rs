use obscur_resources::handle::ResourceHandle;

use super::ResourceOutcome;

/// A resource load queued to run on the next explicit resource poll.
pub(super) struct DeferredLoad {
    pub(super) handle: ResourceHandle,
    pub(super) run: Box<dyn FnOnce() -> ResourceOutcome + Send>,
}
