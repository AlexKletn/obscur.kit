//! The [`GestureArena`] resolving competing recognizers for a pointer sequence
//! (spec §6.16 mobile gestures).

use super::{GestureContender, GestureResolution, RecognizerId, RecognizerState};

/// Resolves competing recognizers contending for a single pointer sequence
/// (spec §6.16).
///
/// Multiple [`GestureContender`]s may claim the same pointer sequence. The
/// arena tracks each contender alongside its current [`RecognizerState`] and
/// applies the arena rule to award the sequence to a single winner:
///
/// - If any contender is [`Recognized`](RecognizerState::Recognized), the
///   winner is the recognized contender with the **highest priority** (ties
///   broken toward the smallest [`RecognizerId`]). System gestures win ties by
///   carrying a high priority.
/// - Otherwise, if there is at least one contender and every contender is
///   terminal but not recognized ([`Failed`](RecognizerState::Failed) /
///   [`Cancelled`](RecognizerState::Cancelled)), the result is
///   [`NoWinner`](GestureResolution::NoWinner).
/// - Otherwise the arena is still
///   [`Pending`](GestureResolution::Pending).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GestureArena {
    contenders: Vec<(GestureContender, RecognizerState)>,
}

impl GestureArena {
    /// Creates an empty arena.
    #[must_use]
    pub fn new() -> Self {
        Self {
            contenders: Vec::new(),
        }
    }

    /// Enters `contender` into the arena in the
    /// [`Possible`](RecognizerState::Possible) state.
    pub fn add(&mut self, contender: GestureContender) {
        self.contenders.push((contender, RecognizerState::Possible));
    }

    /// Updates the tracked state of the contender for `recognizer` to `state`.
    ///
    /// Returns `false` (and changes nothing) if no contender for `recognizer`
    /// is present.
    pub fn mark(&mut self, recognizer: RecognizerId, state: RecognizerState) -> bool {
        for (contender, slot) in &mut self.contenders {
            if contender.recognizer() == recognizer {
                *slot = state;
                return true;
            }
        }
        false
    }

    /// Applies the arena rule and returns the resulting
    /// [`GestureResolution`] without mutating the arena.
    #[must_use]
    pub fn resolve(&self) -> GestureResolution {
        let mut best: Option<(i32, RecognizerId)> = None;
        for (contender, state) in &self.contenders {
            if matches!(state, RecognizerState::Recognized) {
                let candidate = (contender.priority(), contender.recognizer());
                let better = match best {
                    None => true,
                    Some((priority, id)) => {
                        candidate.0 > priority || (candidate.0 == priority && candidate.1 < id)
                    }
                };
                if better {
                    best = Some(candidate);
                }
            }
        }
        if let Some((_, id)) = best {
            return GestureResolution::Winner(id);
        }
        if !self.contenders.is_empty()
            && self.contenders.iter().all(|(_, state)| state.is_terminal())
        {
            return GestureResolution::NoWinner;
        }
        GestureResolution::Pending
    }

    /// Resolves the arena and, if there is a
    /// [`Winner`](GestureResolution::Winner), cancels every other non-terminal
    /// contender (the arena awards the gesture to one recognizer). Returns the
    /// resolution.
    pub fn sweep(&mut self) -> GestureResolution {
        let resolution = self.resolve();
        if let GestureResolution::Winner(winner) = resolution {
            for (contender, state) in &mut self.contenders {
                if contender.recognizer() != winner && !state.is_terminal() {
                    *state = RecognizerState::Cancelled;
                }
            }
        }
        resolution
    }

    /// Returns the contenders and their tracked states.
    #[must_use]
    pub fn contenders(&self) -> &[(GestureContender, RecognizerState)] {
        &self.contenders
    }

    /// Returns the number of contenders.
    #[must_use]
    pub fn len(&self) -> usize {
        self.contenders.len()
    }

    /// Returns whether the arena has no contenders.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.contenders.is_empty()
    }

    /// Removes all contenders.
    pub fn clear(&mut self) {
        self.contenders.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::NodeId;

    fn contender(recognizer: u64, priority: i32) -> GestureContender {
        GestureContender::new(RecognizerId::from_raw(recognizer), NodeId::from_raw(1))
            .with_priority(priority)
    }

    #[test]
    fn empty_arena_is_pending() {
        let arena = GestureArena::new();
        assert!(arena.is_empty());
        assert_eq!(arena.resolve(), GestureResolution::Pending);
    }

    #[test]
    fn still_possible_is_pending() {
        let mut arena = GestureArena::new();
        arena.add(contender(1, 0));
        arena.add(contender(2, 0));
        assert_eq!(arena.len(), 2);
        assert_eq!(arena.resolve(), GestureResolution::Pending);
    }

    #[test]
    fn recognized_wins() {
        let mut arena = GestureArena::new();
        arena.add(contender(1, 0));
        arena.add(contender(2, 0));
        assert!(arena.mark(RecognizerId::from_raw(2), RecognizerState::Recognized));
        assert_eq!(
            arena.resolve(),
            GestureResolution::Winner(RecognizerId::from_raw(2))
        );
    }

    #[test]
    fn mark_unknown_returns_false() {
        let mut arena = GestureArena::new();
        arena.add(contender(1, 0));
        assert!(!arena.mark(RecognizerId::from_raw(99), RecognizerState::Recognized));
    }

    #[test]
    fn highest_priority_recognized_wins() {
        let mut arena = GestureArena::new();
        arena.add(contender(1, 10)); // system-priority
        arena.add(contender(2, 0));
        arena.mark(RecognizerId::from_raw(1), RecognizerState::Recognized);
        arena.mark(RecognizerId::from_raw(2), RecognizerState::Recognized);
        assert_eq!(
            arena.resolve(),
            GestureResolution::Winner(RecognizerId::from_raw(1))
        );
    }

    #[test]
    fn tie_priority_breaks_to_smallest_id() {
        let mut arena = GestureArena::new();
        arena.add(contender(5, 3));
        arena.add(contender(2, 3));
        arena.mark(RecognizerId::from_raw(5), RecognizerState::Recognized);
        arena.mark(RecognizerId::from_raw(2), RecognizerState::Recognized);
        assert_eq!(
            arena.resolve(),
            GestureResolution::Winner(RecognizerId::from_raw(2))
        );
    }

    #[test]
    fn all_failed_is_no_winner() {
        let mut arena = GestureArena::new();
        arena.add(contender(1, 0));
        arena.add(contender(2, 0));
        arena.mark(RecognizerId::from_raw(1), RecognizerState::Failed);
        arena.mark(RecognizerId::from_raw(2), RecognizerState::Cancelled);
        assert_eq!(arena.resolve(), GestureResolution::NoWinner);
    }

    #[test]
    fn one_failed_one_possible_is_pending() {
        let mut arena = GestureArena::new();
        arena.add(contender(1, 0));
        arena.add(contender(2, 0));
        arena.mark(RecognizerId::from_raw(1), RecognizerState::Failed);
        assert_eq!(arena.resolve(), GestureResolution::Pending);
    }

    #[test]
    fn sweep_cancels_other_non_terminal_contenders() {
        let mut arena = GestureArena::new();
        arena.add(contender(1, 0));
        arena.add(contender(2, 0));
        arena.add(contender(3, 0));
        arena.mark(RecognizerId::from_raw(1), RecognizerState::Recognized);
        // Leave 3 already Failed to confirm sweep does not disturb terminals.
        arena.mark(RecognizerId::from_raw(3), RecognizerState::Failed);

        let resolution = arena.sweep();
        assert_eq!(
            resolution,
            GestureResolution::Winner(RecognizerId::from_raw(1))
        );

        let states: Vec<_> = arena
            .contenders()
            .iter()
            .map(|(c, s)| (c.recognizer(), *s))
            .collect();
        assert_eq!(
            states,
            vec![
                (RecognizerId::from_raw(1), RecognizerState::Recognized),
                (RecognizerId::from_raw(2), RecognizerState::Cancelled),
                (RecognizerId::from_raw(3), RecognizerState::Failed),
            ]
        );
    }

    #[test]
    fn sweep_without_winner_leaves_states() {
        let mut arena = GestureArena::new();
        arena.add(contender(1, 0));
        arena.add(contender(2, 0));
        assert_eq!(arena.sweep(), GestureResolution::Pending);
        assert!(
            arena
                .contenders()
                .iter()
                .all(|(_, s)| matches!(s, RecognizerState::Possible))
        );
    }

    #[test]
    fn clear_empties_arena() {
        let mut arena = GestureArena::new();
        arena.add(contender(1, 0));
        arena.clear();
        assert!(arena.is_empty());
    }
}
