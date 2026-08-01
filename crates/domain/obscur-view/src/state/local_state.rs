#![doc = include_str!("local_state.doc/_module.md")]

use crate::state::{ReadSignal, StateGeneration};

#[doc = include_str!("local_state.doc/LocalState.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LocalState<T> {
    value: T,
    generation: StateGeneration,
}

impl<T> LocalState<T> {
    #[doc = include_str!("local_state.doc/new.md")]
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self {
            value,
            generation: StateGeneration::INITIAL,
        }
    }

    #[doc = include_str!("local_state.doc/get.md")]
    #[must_use]
    pub const fn get(&self) -> &T {
        &self.value
    }

    #[doc = include_str!("local_state.doc/generation.md")]
    #[must_use]
    pub const fn generation(&self) -> StateGeneration {
        self.generation
    }

    #[doc = include_str!("local_state.doc/set.md")]
    pub fn set(&mut self, value: T) {
        self.value = value;
        self.bump();
    }

    #[doc = include_str!("local_state.doc/update.md")]
    pub fn update(&mut self, f: impl FnOnce(&mut T)) {
        f(&mut self.value);
        self.bump();
    }

    #[doc = include_str!("local_state.doc/bump.md")]
    fn bump(&mut self) {
        self.generation = self.generation.next();
    }
}

impl<T: Clone> LocalState<T> {
    #[doc = include_str!("local_state.doc/read.md")]
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
