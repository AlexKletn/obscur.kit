use std::{sync::mpsc::Receiver, time::Instant};

use obscur_resources::{handle::ResourceHandle, state::ResourceVersion};

use super::ResourceOutcome;

/// A dispatched resource request and its one-shot completion channel.
pub(super) struct PendingLoad {
    pub(super) key: Box<str>,
    pub(super) handle: ResourceHandle,
    pub(super) version: ResourceVersion,
    pub(super) submitted: Instant,
    pub(super) result: Receiver<ResourceOutcome>,
}
