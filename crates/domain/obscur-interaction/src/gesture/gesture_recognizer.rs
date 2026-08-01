#![doc = include_str!("gesture_recognizer.doc/_module.md")]

#[doc = include_str!("gesture_recognizer.doc/RecognizerId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RecognizerId(u64);

impl RecognizerId {
    #[doc = include_str!("gesture_recognizer.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[doc = include_str!("gesture_recognizer.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[doc = include_str!("gesture_recognizer.doc/GestureKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GestureKind {
    #[doc = include_str!("gesture_recognizer.doc/Tap.md")]
    Tap,
    #[doc = include_str!("gesture_recognizer.doc/DoubleTap.md")]
    DoubleTap,
    #[doc = include_str!("gesture_recognizer.doc/LongPress.md")]
    LongPress,
    #[doc = include_str!("gesture_recognizer.doc/Pan.md")]
    Pan,
    #[doc = include_str!("gesture_recognizer.doc/Swipe.md")]
    Swipe,
    #[doc = include_str!("gesture_recognizer.doc/Pinch.md")]
    Pinch,
    #[doc = include_str!("gesture_recognizer.doc/Rotate.md")]
    Rotate,
}

#[doc = include_str!("gesture_recognizer.doc/RecognizerState.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecognizerState {
    #[doc = include_str!("gesture_recognizer.doc/Possible.md")]
    Possible,
    #[doc = include_str!("gesture_recognizer.doc/Began.md")]
    Began,
    #[doc = include_str!("gesture_recognizer.doc/Recognized.md")]
    Recognized,
    #[doc = include_str!("gesture_recognizer.doc/Failed.md")]
    Failed,
    #[doc = include_str!("gesture_recognizer.doc/Cancelled.md")]
    Cancelled,
}

impl RecognizerState {
    #[doc = include_str!("gesture_recognizer.doc/is_terminal.md")]
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Recognized | Self::Failed | Self::Cancelled)
    }

    #[doc = include_str!("gesture_recognizer.doc/is_active.md")]
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Possible | Self::Began)
    }
}

#[doc = include_str!("gesture_recognizer.doc/GestureRecognizer.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GestureRecognizer {
    id: RecognizerId,
    kind: GestureKind,
    state: RecognizerState,
}

impl GestureRecognizer {
    #[doc = include_str!("gesture_recognizer.doc/new.md")]
    #[must_use]
    pub const fn new(id: RecognizerId, kind: GestureKind) -> Self {
        Self {
            id,
            kind,
            state: RecognizerState::Possible,
        }
    }

    #[doc = include_str!("gesture_recognizer.doc/begin.md")]
    pub fn begin(&mut self) {
        if matches!(self.state, RecognizerState::Possible) {
            self.state = RecognizerState::Began;
        }
    }

    #[doc = include_str!("gesture_recognizer.doc/recognize.md")]
    pub fn recognize(&mut self) {
        if !self.state.is_terminal() {
            self.state = RecognizerState::Recognized;
        }
    }

    #[doc = include_str!("gesture_recognizer.doc/fail.md")]
    pub fn fail(&mut self) {
        if !self.state.is_terminal() {
            self.state = RecognizerState::Failed;
        }
    }

    #[doc = include_str!("gesture_recognizer.doc/cancel.md")]
    pub fn cancel(&mut self) {
        if !self.state.is_terminal() {
            self.state = RecognizerState::Cancelled;
        }
    }

    #[doc = include_str!("gesture_recognizer.doc/id.md")]
    #[must_use]
    pub const fn id(self) -> RecognizerId {
        self.id
    }

    #[doc = include_str!("gesture_recognizer.doc/kind.md")]
    #[must_use]
    pub const fn kind(self) -> GestureKind {
        self.kind
    }

    #[doc = include_str!("gesture_recognizer.doc/state.md")]
    #[must_use]
    pub const fn state(self) -> RecognizerState {
        self.state
    }

    #[doc = include_str!("gesture_recognizer.doc/is_recognized.md")]
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
    fn recognizer_id_roundtrips() {
        assert_eq!(RecognizerId::from_raw(7).raw(), 7);
    }

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
