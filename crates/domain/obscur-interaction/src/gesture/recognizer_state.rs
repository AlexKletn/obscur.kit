#[doc = include_str!("../../docs/api/gesture/gesture_recognizer.doc/RecognizerState.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecognizerState {
    /// Contending; the pointer sequence may still become this gesture.
    Possible,
    /// A continuous gesture has started but not yet fully recognized.
    Began,
    /// The gesture was recognized. Terminal.
    Recognized,
    /// The gesture was ruled out. Terminal.
    Failed,
    /// The gesture was cancelled (e.g. lost the arena). Terminal.
    Cancelled,
}

impl RecognizerState {
    /// Returns whether this is a terminal state
    /// ([`Recognized`](Self::Recognized), [`Failed`](Self::Failed) or
    /// [`Cancelled`](Self::Cancelled)).
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Recognized | Self::Failed | Self::Cancelled)
    }

    /// Returns whether this is a still-contending state
    /// ([`Possible`](Self::Possible) or [`Began`](Self::Began)).
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Possible | Self::Began)
    }
}

#[cfg(test)]
mod tests {
    use super::RecognizerState;

    #[test]
    fn state_predicates() {
        assert!(RecognizerState::Possible.is_active());
        assert!(RecognizerState::Began.is_active());
        assert!(!RecognizerState::Recognized.is_active());
        assert!(RecognizerState::Recognized.is_terminal());
        assert!(RecognizerState::Failed.is_terminal());
        assert!(RecognizerState::Cancelled.is_terminal());
        assert!(!RecognizerState::Possible.is_terminal());
    }
}
