//! The [`GestureRecognizer`] and its lifecycle state (spec §6.16 mobile
//! gestures).

/// A stable identifier for a [`GestureRecognizer`] within a gesture arena
/// (spec §6.16).
///
/// This is a local newtype over an opaque `u64`; it is distinct from
/// [`NodeId`](obscur_identity::NodeId) and only carries meaning within the
/// interaction layer's gesture arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RecognizerId(u64);

impl RecognizerId {
    /// Creates a recognizer id from a raw `u64`.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw `u64` backing this id.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// The kind of gesture a recognizer detects (spec §6.16).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GestureKind {
    /// A single tap.
    Tap,
    /// Two taps in quick succession.
    DoubleTap,
    /// A press held past a duration threshold.
    LongPress,
    /// A one-or-more-finger drag.
    Pan,
    /// A quick directional flick.
    Swipe,
    /// A two-finger scale.
    Pinch,
    /// A two-finger rotation.
    Rotate,
}

/// The lifecycle state of a [`GestureRecognizer`] (spec §6.16).
///
/// Recognition proceeds `Possible` → `Began` → `Recognized`, or short-circuits
/// to `Failed` / `Cancelled`. A recognizer may also recognize directly from
/// `Possible` (discrete gestures such as taps). The three terminal states
/// ([`Recognized`](Self::Recognized), [`Failed`](Self::Failed),
/// [`Cancelled`](Self::Cancelled)) are **sticky**: once reached, the state no
/// longer changes.
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

/// A gesture recognizer with a lifecycle [`RecognizerState`] (spec §6.16).
///
/// A recognizer observes a pointer sequence and drives its state toward a
/// terminal outcome. Recognizers contend inside a
/// [`GestureArena`](super::GestureArena); the arena awards the sequence to a
/// single winner and cancels the rest.
///
/// State transitions (terminal states are sticky — no-ops once reached):
///
/// - [`begin`](Self::begin): `Possible` → `Began`.
/// - [`recognize`](Self::recognize): `Possible` or `Began` → `Recognized`.
/// - [`fail`](Self::fail): any non-terminal → `Failed`.
/// - [`cancel`](Self::cancel): any non-terminal → `Cancelled`.
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
