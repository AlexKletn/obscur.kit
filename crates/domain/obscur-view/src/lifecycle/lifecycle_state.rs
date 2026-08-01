#![doc = include_str!("lifecycle_state.doc/_module.md")]

#[doc = include_str!("lifecycle_state.doc/LifecycleState.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifecycleState {
    #[doc = include_str!("lifecycle_state.doc/Created.md")]
    Created,
    #[doc = include_str!("lifecycle_state.doc/Mounted.md")]
    Mounted,
    #[doc = include_str!("lifecycle_state.doc/Updating.md")]
    Updating,
    #[doc = include_str!("lifecycle_state.doc/Unmounted.md")]
    Unmounted,
}

impl LifecycleState {
    #[doc = include_str!("lifecycle_state.doc/is_active.md")]
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Mounted | Self::Updating)
    }

    #[doc = include_str!("lifecycle_state.doc/is_mounted.md")]
    #[must_use]
    pub const fn is_mounted(self) -> bool {
        self.is_active()
    }

    #[doc = include_str!("lifecycle_state.doc/is_unmounted.md")]
    #[must_use]
    pub const fn is_unmounted(self) -> bool {
        matches!(self, Self::Unmounted)
    }
}

#[cfg(test)]
mod tests {
    use super::LifecycleState;

    #[test]
    fn active_only_when_mounted_or_updating() {
        assert!(!LifecycleState::Created.is_active());
        assert!(LifecycleState::Mounted.is_active());
        assert!(LifecycleState::Updating.is_active());
        assert!(!LifecycleState::Unmounted.is_active());
    }

    #[test]
    fn mounted_matches_active() {
        for state in [
            LifecycleState::Created,
            LifecycleState::Mounted,
            LifecycleState::Updating,
            LifecycleState::Unmounted,
        ] {
            assert_eq!(state.is_mounted(), state.is_active());
        }
    }

    #[test]
    fn unmounted_is_terminal() {
        assert!(LifecycleState::Unmounted.is_unmounted());
        assert!(!LifecycleState::Mounted.is_unmounted());
    }
}
