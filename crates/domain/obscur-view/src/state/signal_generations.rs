//! `SignalGenerations`: the current version of every observable source.

use std::collections::HashMap;

use obscur_identity::SignalId;

use crate::state::StateGeneration;

/// The current [`StateGeneration`] of each observable source (spec §3.1).
///
/// State enters the UI through typed props, local component state, and
/// **read-only signal/observable sources**. This is the version book for the
/// third: an application bumps a source when the value behind it changes, and a
/// component records which sources its render read. Comparing the two is what
/// makes "dependency tracking invalidates only consumers of changed values" true
/// — a source that did not advance invalidates nobody, and a component that read
/// nothing is never invalidated.
///
/// # It holds versions, not values
///
/// Deliberately: the toolkit provides no store, and a component owns its own
/// data (§2.1). A `SignalId` is an opaque name for "something the app can
/// change"; what it names, and where that value lives, is entirely the
/// application's business. This only answers *has it changed since you looked*.
///
/// An unknown source reads as [`StateGeneration::INITIAL`], so tracking a source
/// nobody has bumped yet is well-defined rather than an error.
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

    /// Advances `source` to a new generation and returns it.
    ///
    /// Called when the value behind the source changed. The toolkit does not diff
    /// values to decide whether a change occurred — bumping *is* the statement
    /// that it did, matching [`LocalState`](crate::state::LocalState)'s rule.
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
