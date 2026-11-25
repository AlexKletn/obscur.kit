//! `SignalGenerations`: the current version of every observable source.

use std::collections::HashMap;

use obscur_identity::SignalId;

use crate::state::StateGeneration;

#[doc = include_str!("../../docs/api/state/signal_generations.doc/SignalGenerations.md")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SignalGenerations {
    /// Sources that have been bumped at least once. Absent == `INITIAL`.
    versions: HashMap<SignalId, StateGeneration>,
}

impl SignalGenerations {
    /// Creates a book in which every source is at [`StateGeneration::INITIAL`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The current generation of `source`.
    #[must_use]
    pub fn generation(&self, source: SignalId) -> StateGeneration {
        self.versions
            .get(&source)
            .copied()
            .unwrap_or(StateGeneration::INITIAL)
    }

    #[doc = include_str!("../../docs/api/state/signal_generations.doc/bump.md")]
    pub fn bump(&mut self, source: SignalId) -> StateGeneration {
        let next = self.generation(source).next();
        self.versions.insert(source, next);
        next
    }

    /// How many sources have ever been bumped.
    #[must_use]
    pub fn len(&self) -> usize {
        self.versions.len()
    }

    /// Whether no source has ever been bumped.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.versions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::SignalGenerations;
    use crate::state::StateGeneration;
    use obscur_identity::SignalId;

    fn source(raw: u64) -> SignalId {
        SignalId::from_raw(raw)
    }

    #[test]
    fn an_untouched_source_reads_as_the_initial_generation() {
        let generations = SignalGenerations::new();
        assert_eq!(generations.generation(source(1)), StateGeneration::INITIAL);
        assert!(generations.is_empty());
    }

    #[test]
    fn bumping_advances_only_that_source() {
        let mut generations = SignalGenerations::new();
        let bumped = generations.bump(source(1));

        assert_eq!(generations.generation(source(1)), bumped);
        assert!(bumped.is_newer_than(StateGeneration::INITIAL));
        // The whole point: an unrelated source is untouched, so its consumers
        // are not invalidated.
        assert_eq!(generations.generation(source(2)), StateGeneration::INITIAL);
    }

    #[test]
    fn each_bump_yields_a_strictly_newer_generation() {
        let mut generations = SignalGenerations::new();
        let first = generations.bump(source(1));
        let second = generations.bump(source(1));
        assert!(second.is_newer_than(first));
    }
}
