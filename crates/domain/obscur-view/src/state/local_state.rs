//! `LocalState`: a component's owned, mutable local view state.

use crate::state::{ReadSignal, StateGeneration};

/// A component's owned, mutable local view state.
///
/// Per spec §2.1/§10 a component *may* own local view state; this is that
/// primitive. Because the component owns the value outright, mutation is
/// expressed through `&mut self` — plain Rust ownership — rather than interior
/// mutability (`RefCell`/`Rc`) or any global store. Every mutation advances a
/// [`StateGeneration`] so downstream consumers can cheaply detect the change.
///
/// State only ever flows *out* as a read-only [`ReadSignal`] via
/// [`LocalState::read`]; there is no shared writable handle, keeping the
/// "values flow down, actions flow up" discipline intact.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LocalState<T> {
    value: T,
    generation: StateGeneration,
}

impl<T> LocalState<T> {
    /// Creates local state seeded with `value` at the initial generation.
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self {
            value,
            generation: StateGeneration::INITIAL,
        }
    }

    /// Borrows the current value without affecting the generation.
    #[must_use]
    pub const fn get(&self) -> &T {
        &self.value
    }

    /// Returns the current generation. It advances on every mutation.
    #[must_use]
    pub const fn generation(&self) -> StateGeneration {
        self.generation
    }

    /// Replaces the value and advances the generation.
    pub fn set(&mut self, value: T) {
        self.value = value;
        self.bump();
    }

    /// Mutates the value in place through `f` and advances the generation.
    ///
    /// The generation advances unconditionally, matching the spec's "mutation
    /// bumps it" rule: the toolkit does not diff values to decide whether a
    /// change occurred.
    pub fn update(&mut self, f: impl FnOnce(&mut T)) {
        f(&mut self.value);
        self.bump();
    }

    /// Advances the change generation. Wraps on overflow via [`StateGeneration`].
    fn bump(&mut self) {
        self.generation = self.generation.next();
    }
}

impl<T: Clone> LocalState<T> {
    /// Produces a read-only [`ReadSignal`] snapshot of the current value and
    /// generation.
    ///
    /// This is the only way state leaves `LocalState`, and it is read-only:
    /// consumers receive a value they cannot write back through. Re-calling
    /// `read` after a mutation yields a snapshot with a newer generation.
    #[must_use]
    pub fn read(&self) -> ReadSignal<T> {
        ReadSignal::new(self.value.clone(), self.generation)
    }
}

#[cfg(test)]
mod tests {
    use super::LocalState;
    use crate::state::StateGeneration;

    #[test]
    fn new_starts_at_initial_generation() {
        let state = LocalState::new(10_i32);
        assert_eq!(*state.get(), 10);
        assert_eq!(state.generation(), StateGeneration::INITIAL);
    }

    #[test]
    fn set_replaces_and_bumps() {
        let mut state = LocalState::new(1_u8);
        let before = state.generation();
        state.set(2);
        assert_eq!(*state.get(), 2);
        assert!(state.generation().is_newer_than(before));
    }

    #[test]
    fn update_mutates_in_place_and_bumps() {
        let mut state = LocalState::new(vec![1, 2]);
        let before = state.generation();
        state.update(|v| v.push(3));
        assert_eq!(*state.get(), vec![1, 2, 3]);
        assert!(state.generation().is_newer_than(before));
    }

    #[test]
    fn read_yields_matching_snapshot() {
        let mut state = LocalState::new(5_u32);
        let s0 = state.read();
        assert_eq!(*s0.get(), 5);

        state.set(6);
        let s1 = state.read();
        assert_eq!(*s1.get(), 6);
        assert!(s1.changed_since(s0.generation()));
    }
}
