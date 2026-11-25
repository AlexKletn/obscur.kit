//! Observable cancellation flag.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// A cheaply cloneable handle observed by scoped tasks to learn they should
/// stop. Produced by a [`CancellationSource`](super::CancellationSource).
#[derive(Debug, Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    /// Returns whether cancellation has been requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    pub(crate) fn request(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}
