#![doc = include_str!("gesture_arena.doc/_module.md")]

use super::{GestureContender, GestureResolution, RecognizerId, RecognizerState};

#[doc = include_str!("gesture_arena.doc/GestureArena.md")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GestureArena {
    contenders: Vec<(GestureContender, RecognizerState)>,
}

impl GestureArena {
    #[doc = include_str!("gesture_arena.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            contenders: Vec::new(),
        }
    }

    #[doc = include_str!("gesture_arena.doc/add.md")]
    pub fn add(&mut self, contender: GestureContender) {
        self.contenders.push((contender, RecognizerState::Possible));
    }

    #[doc = include_str!("gesture_arena.doc/mark.md")]
    pub fn mark(&mut self, recognizer: RecognizerId, state: RecognizerState) -> bool {
        for (contender, slot) in &mut self.contenders {
            if contender.recognizer() == recognizer {
                *slot = state;
                return true;
            }
        }
        false
    }

    #[doc = include_str!("gesture_arena.doc/resolve.md")]
    #[must_use]
    pub fn resolve(&self) -> GestureResolution {
        // Iterate in insertion order. A candidate wins when its priority is
        // strictly greater, or equal — the latter lets the most recently entered
        // contender take a tie, so a system gesture entered after the app's
        // recognizers wins by default (§6.16).
        let mut best: Option<(i32, RecognizerId)> = None;
        for (contender, state) in &self.contenders {
            if matches!(state, RecognizerState::Recognized) {
                let priority = contender.priority();
                let take = match best {
                    None => true,
                    Some((best_priority, _)) => priority >= best_priority,
                };
                if take {
                    best = Some((priority, contender.recognizer()));
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

    #[doc = include_str!("gesture_arena.doc/sweep.md")]
    pub fn sweep(&mut self) -> GestureResolution {
        let resolution = self.resolve();
        if let GestureResolution::Winner(winner) = resolution {
            for (contender, state) in &mut self.contenders {
                // Every loser yields the sequence to the winner. A loser that
                // had itself reached `Recognized` is revoked to `Cancelled` — it
                // must receive no further updates (§6.16) — while contenders that
                // already Failed or Cancelled are left as they are.
                let already_out =
                    matches!(state, RecognizerState::Failed | RecognizerState::Cancelled);
                if contender.recognizer() != winner && !already_out {
                    *state = RecognizerState::Cancelled;
                }
            }
        }
        resolution
    }

    #[doc = include_str!("gesture_arena.doc/contenders.md")]
    #[must_use]
    pub fn contenders(&self) -> &[(GestureContender, RecognizerState)] {
        &self.contenders
    }

    #[doc = include_str!("gesture_arena.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.contenders.len()
    }

    #[doc = include_str!("gesture_arena.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.contenders.is_empty()
    }

    #[doc = include_str!("gesture_arena.doc/clear.md")]
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
    fn tie_priority_breaks_to_most_recently_entered() {
        // Equal priority: the contender entered later (id 2, added second) takes
        // the tie, so a system gesture entered after app recognizers wins.
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
