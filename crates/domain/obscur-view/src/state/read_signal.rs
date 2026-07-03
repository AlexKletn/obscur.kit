//! `ReadSignal`: a read-only value source. Values flow down; there is no setter.

use crate::state::StateGeneration;

/// Read-only snapshot of a value together with the [`StateGeneration`] at which
/// it was produced.
///
/// `ReadSignal<T>` is the "values flow down" half of the data-flow model: it
/// hands consumers the current value and a change token, but exposes **no**
/// mutation API, so there is no magic two-way binding. A component obtains one
/// from its own [`LocalState`](crate::state::LocalState) (or, later, from an
/// upstream source) and passes it down as a prop.
///
/// Because there is no reactive runtime yet, a `ReadSignal` is a lightweight,
/// self-contained snapshot rather than a live handle: it carries the value by
/// ownership and its generation by value. Re-reading an updated source means
/// producing a fresh snapshot with a newer generation; a consumer detects the
/// change with [`ReadSignal::changed_since`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ReadSignal<T> {
    value: T,
    generation: StateGeneration,
}

impl<T> ReadSignal<T> {
    /// Builds a read-only signal snapshot from a value and the generation that
    /// produced it.
    #[must_use]
    pub const fn new(value: T, generation: StateGeneration) -> Self {
        Self { value, generation }
    }

    /// Borrows the current value.
    #[must_use]
    pub const fn get(&self) -> &T {
        &self.value
    }

    /// Returns the generation this snapshot was taken at.
    #[must_use]
    pub const fn generation(&self) -> StateGeneration {
        self.generation
    }

    /// Returns `true` if this snapshot is newer than the generation the consumer
    /// last observed, i.e. the value changed since `last_seen`.
    ///
    /// Equal generations report `false`, so unchanged sources trigger no work.
    #[must_use]
    pub fn changed_since(&self, last_seen: StateGeneration) -> bool {
        self.generation.is_newer_than(last_seen)
    }

    /// Consumes the snapshot and returns the owned value.
    #[must_use]
    pub fn into_value(self) -> T {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::ReadSignal;
    use crate::state::StateGeneration;

    #[test]
    fn get_returns_snapshot_value() {
        let sig = ReadSignal::new(42_u32, StateGeneration::INITIAL);
        assert_eq!(*sig.get(), 42);
        assert_eq!(sig.generation(), StateGeneration::INITIAL);
    }

    #[test]
    fn changed_since_tracks_generation() {
        let seen = StateGeneration::INITIAL;
        let newer = ReadSignal::new("v2", seen.next());
        assert!(newer.changed_since(seen));

        let unchanged = ReadSignal::new("v1", seen);
        assert!(!unchanged.changed_since(seen));
    }

    #[test]
    fn into_value_unwraps() {
        let sig = ReadSignal::new(vec![1, 2, 3], StateGeneration::from_raw(5));
        assert_eq!(sig.into_value(), vec![1, 2, 3]);
    }
}
