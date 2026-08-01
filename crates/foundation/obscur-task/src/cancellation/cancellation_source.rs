#![doc = include_str!("cancellation_source.doc/_module.md")]

use super::CancellationToken;

#[doc = include_str!("cancellation_source.doc/CancellationSource.md")]
#[derive(Debug, Clone, Default)]
pub struct CancellationSource {
    token: CancellationToken,
}

impl CancellationSource {
    #[doc = include_str!("cancellation_source.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("cancellation_source.doc/token.md")]
    #[must_use]
    pub fn token(&self) -> CancellationToken {
        self.token.clone()
    }

    #[doc = include_str!("cancellation_source.doc/cancel.md")]
    pub fn cancel(&self) {
        self.token.request();
    }
}
