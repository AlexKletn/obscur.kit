//! The [`GestureRecognizer`] and its lifecycle state (spec §6.16 mobile
//! gestures).

use super::{GestureKind, RecognizerId, RecognizerState};

#[doc = include_str!("../../docs/api/gesture/gesture_recognizer.doc/GestureRecognizer.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GestureRecognizer {
    id: RecognizerId,
    kind: GestureKind,
    state: RecognizerState,
}

impl GestureRecognizer {
    /// Creates a recognizer of `kind` identified by `id`, in the
    /// [`Possible`](RecognizerState::Possible) state.
    #[must_use]
    pub const fn new(id: RecognizerId, kind: GestureKind) -> Self {
        Self {
            id,
            kind,
            state: RecognizerState::Possible,
        }
    }

    /// Transitions [`Possible`](RecognizerState::Possible) →
    /// [`Began`](RecognizerState::Began). No-op in any other state.
    pub fn begin(&mut self) {
        if matches!(self.state, RecognizerState::Possible) {
            self.state = RecognizerState::Began;
        }
    }

    /// Transitions [`Possible`](RecognizerState::Possible) or
    /// [`Began`](RecognizerState::Began) →
    /// [`Recognized`](RecognizerState::Recognized). No-op if already terminal.
    pub fn recognize(&mut self) {
        if !self.state.is_terminal() {
            self.state = RecognizerState::Recognized;
        }
    }

    /// Transitions any non-terminal state →
    /// [`Failed`](RecognizerState::Failed). No-op if already terminal.
    pub fn fail(&mut self) {
        if !self.state.is_terminal() {
            self.state = RecognizerState::Failed;
        }
    }

    /// Transitions any non-terminal state →
    /// [`Cancelled`](RecognizerState::Cancelled). No-op if already terminal.
    pub fn cancel(&mut self) {
        if !self.state.is_terminal() {
            self.state = RecognizerState::Cancelled;
        }
    }

    /// Returns this recognizer's id.
    #[must_use]
    pub const fn id(self) -> RecognizerId {
        self.id
    }

    /// Returns the kind of gesture this recognizer detects.
    #[must_use]
    pub const fn kind(self) -> GestureKind {
        self.kind
    }

    /// Returns the current lifecycle state.
    #[must_use]
    pub const fn state(self) -> RecognizerState {
        self.state
    }

    /// Returns whether the recognizer has reached
    /// [`Recognized`](RecognizerState::Recognized).
    #[must_use]
    pub const fn is_recognized(self) -> bool {
        matches!(self.state, RecognizerState::Recognized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec() -> GestureRecognizer {
        GestureRecognizer::new(RecognizerId::from_raw(1), GestureKind::Pan)
    }

    #[test]
    fn new_starts_possible() {
        let r = rec();
        assert_eq!(r.state(), RecognizerState::Possible);
        assert_eq!(r.id(), RecognizerId::from_raw(1));
        assert_eq!(r.kind(), GestureKind::Pan);
        assert!(!r.is_recognized());
    }

    #[test]
    fn begin_then_recognize() {
        let mut r = rec();
        r.begin();
        assert_eq!(r.state(), RecognizerState::Began);
        r.recognize();
        assert_eq!(r.state(), RecognizerState::Recognized);
        assert!(r.is_recognized());
    }

    #[test]
    fn recognize_directly_from_possible() {
        let mut r = rec();
        r.recognize();
        assert!(r.is_recognized());
    }

    #[test]
    fn fail_and_cancel() {
        let mut a = rec();
        a.fail();
        assert_eq!(a.state(), RecognizerState::Failed);
        let mut b = rec();
        b.cancel();
        assert_eq!(b.state(), RecognizerState::Cancelled);
    }

    #[test]
    fn terminal_states_are_sticky() {
        let mut r = rec();
        r.recognize();
        r.fail();
        r.cancel();
        r.begin();
        assert_eq!(r.state(), RecognizerState::Recognized);

        let mut f = rec();
        f.fail();
        f.recognize();
        assert_eq!(f.state(), RecognizerState::Failed);
    }

    #[test]
    fn begin_is_noop_when_not_possible() {
        let mut r = rec();
        r.recognize();
        r.begin();
        assert_eq!(r.state(), RecognizerState::Recognized);
    }
}
