//! The [`LongPressRecognizer`] — a contact held still past a threshold (spec
//! §6.16 `LongPress`).

use super::{GestureKind, GestureRecognizer, RecognizerId, RecognizerState};

#[doc = include_str!("../../docs/api/gesture/long_press_recognizer.doc/LongPressRecognizer.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LongPressRecognizer {
    recognizer: GestureRecognizer,
    held: f32,
    threshold: f32,
}

impl LongPressRecognizer {
    /// Arms a long press that recognizes after `threshold` seconds of a contact
    /// held still.
    #[must_use]
    pub fn new(id: RecognizerId, threshold: f32) -> Self {
        Self {
            recognizer: GestureRecognizer::new(id, GestureKind::LongPress),
            held: 0.0,
            threshold,
        }
    }

    #[doc = include_str!("../../docs/api/gesture/long_press_recognizer.doc/advance.md")]
    pub fn advance(&mut self, seconds: f32) -> bool {
        if self.recognizer.state().is_terminal() || !seconds.is_finite() || seconds <= 0.0 {
            return false;
        }
        self.held += seconds;
        if self.held < self.threshold {
            return false;
        }
        self.recognizer.recognize();
        true
    }

    #[doc = include_str!("../../docs/api/gesture/long_press_recognizer.doc/moved.md")]
    pub fn moved(&mut self, distance: f32, slop: f32) {
        if distance > slop {
            self.recognizer.fail();
        }
    }

    /// Fails the long press: the contact lifted before the threshold, so it was a
    /// tap.
    pub fn end(&mut self) {
        self.recognizer.fail();
    }

    /// Cancels the long press — the system took the sequence, or another
    /// recognizer won it.
    pub fn cancel(&mut self) {
        self.recognizer.cancel();
    }

    /// Whether the hold has been recognized.
    #[must_use]
    pub fn is_recognized(&self) -> bool {
        self.recognizer.state() == RecognizerState::Recognized
    }

    /// Whether the hold is still a candidate — neither recognized nor ruled out.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.recognizer.state() == RecognizerState::Possible
    }

    /// The recognizer's lifecycle state.
    #[must_use]
    pub fn state(&self) -> RecognizerState {
        self.recognizer.state()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOLD: f32 = 0.5;
    const SLOP: f32 = 10.0;

    fn recognizer() -> LongPressRecognizer {
        LongPressRecognizer::new(RecognizerId::from_raw(1), HOLD)
    }

    #[test]
    fn a_contact_held_past_the_threshold_is_recognized() {
        let mut press = recognizer();
        assert!(!press.advance(0.3), "not yet");
        assert!(press.is_pending());
        assert!(press.advance(0.3), "crossed the threshold");
        assert!(press.is_recognized());
    }

    /// The step that crosses reports once. A consumer ticks every frame, and a
    /// long press that kept reporting would open a context menu sixty times a
    /// second for as long as the finger stayed down.
    #[test]
    fn recognition_is_reported_exactly_once() {
        let mut press = recognizer();
        assert!(press.advance(0.6));
        assert!(!press.advance(0.1));
        assert!(!press.advance(1.0));
        assert!(press.is_recognized(), "and it stays recognized");
    }

    #[test]
    fn a_contact_that_lifts_first_is_a_tap_not_a_hold() {
        let mut press = recognizer();
        press.advance(0.2);
        press.end();
        assert_eq!(press.state(), RecognizerState::Failed);
        assert!(
            !press.advance(1.0),
            "a lifted contact cannot still recognize"
        );
    }

    #[test]
    fn a_contact_that_wanders_fails() {
        let mut press = recognizer();
        press.advance(0.2);
        press.moved(11.0, SLOP);
        assert_eq!(press.state(), RecognizerState::Failed);
        assert!(!press.advance(1.0));
    }

    #[test]
    fn jitter_within_the_slop_does_not_fail_it() {
        // A fingertip rolls while it presses; a hold that any tremor killed would
        // be unreachable.
        let mut press = recognizer();
        press.moved(3.0, SLOP);
        press.moved(9.0, SLOP);
        assert!(press.is_pending());
        assert!(press.advance(0.6));
    }

    #[test]
    fn a_cancelled_hold_never_recognizes() {
        let mut press = recognizer();
        press.cancel();
        assert_eq!(press.state(), RecognizerState::Cancelled);
        assert!(!press.advance(1.0));
    }

    #[test]
    fn the_hold_accumulates_across_steps() {
        // Frames are not one long step: ten 60ms ticks must count the same as one
        // 600ms one.
        let mut press = recognizer();
        let mut fired = false;
        for _ in 0..10 {
            fired |= press.advance(0.06);
        }
        assert!(fired, "0.6s of ticks crossed a 0.5s threshold");
    }
}
