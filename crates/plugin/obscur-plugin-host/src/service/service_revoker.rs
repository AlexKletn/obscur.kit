use std::{
    future::{Future, poll_fn},
    sync::{Arc, atomic::Ordering},
    task::Poll,
};

use super::revocation::Revocation;

/// Host control for cancelling calls at a registration, session, or handle.
#[derive(Clone)]
pub struct ServiceRevoker(Arc<Revocation>);

impl ServiceRevoker {
    pub(super) fn new() -> Self {
        Self(Arc::new(Revocation::default()))
    }

    /// Invalidates the associated registration, grant, or session and wakes pending dispatches.
    /// Returns true for the first revocation.
    pub fn revoke(&self) -> bool {
        if self.0.revoked.swap(true, Ordering::AcqRel) {
            return false;
        }
        self.0.changed.notify(usize::MAX);
        true
    }

    /// Reports whether this control has been revoked.
    #[must_use]
    pub fn is_revoked(&self) -> bool {
        self.0.revoked.load(Ordering::Acquire)
    }

    pub(super) fn same_target(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub(super) async fn cancelled(&self) {
        let listener = self.0.changed.listen();
        if !self.is_revoked() {
            listener.await;
        }
    }
}

pub(super) async fn until_cancelled<T>(
    task: impl Future<Output = T>,
    cancel: impl Future<Output = ()>,
    cancelled: impl Fn() -> T,
) -> T {
    let mut cancel = std::pin::pin!(cancel);
    let mut task = std::pin::pin!(task);
    poll_fn(|context| {
        if cancel.as_mut().poll(context).is_ready() {
            return Poll::Ready(cancelled());
        }
        task.as_mut().poll(context)
    })
    .await
}
