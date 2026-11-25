//! Owner side of a cancellation signal.

use super::CancellationToken;

/// The owning side of a cancellation signal. Dropping or cancelling it flips
/// every derived [`CancellationToken`].
#[derive(Debug, Clone, Default)]
pub struct CancellationSource {
    token: CancellationToken,
}

impl CancellationSource {
    /// Creates a fresh, un-cancelled source.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a token observing this source.
    #[must_use]
    pub fn token(&self) -> CancellationToken {
        self.token.clone()
    }

    /// Requests cancellation of all derived tokens.
    pub fn cancel(&self) {
        self.token.request();
    }
}
