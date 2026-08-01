#![doc = include_str!("cancellation_token.doc/_module.md")]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[doc = include_str!("cancellation_token.doc/CancellationToken.md")]
#[derive(Debug, Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    #[doc = include_str!("cancellation_token.doc/is_cancelled.md")]
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    pub(crate) fn request(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}
