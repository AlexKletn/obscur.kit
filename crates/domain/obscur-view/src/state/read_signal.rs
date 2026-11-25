//! `ReadSignal`: a read-only value source. Values flow down; there is no setter.

use crate::state::StateGeneration;

#[doc = include_str!("../../docs/api/state/read_signal.doc/ReadSignal.md")]
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

    #[doc = include_str!("../../docs/api/state/read_signal.doc/changed_since.md")]
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
