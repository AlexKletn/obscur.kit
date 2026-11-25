//! `LifecycleState`: the phase a component instance is currently in.

#[doc = include_str!("../../docs/api/lifecycle/lifecycle_state.doc/LifecycleState.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifecycleState {
    /// The instance has been constructed but not yet mounted; it has produced
    /// no view and run no `on_mount` hook.
    Created,
    /// The instance is mounted and settled: its subscriptions are live and it
    /// is ready to render.
    Mounted,
    /// The instance is applying an update (new props or invalidated inputs)
    /// and its `on_update` hook may run.
    Updating,
    /// The instance has been torn down; its scoped subscriptions and local
    /// state have been released.
    Unmounted,
}

impl LifecycleState {
    #[doc = include_str!("../../docs/api/lifecycle/lifecycle_state.doc/is_active.md")]
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Mounted | Self::Updating)
    }

    /// Returns whether the instance has been mounted at least once and not yet
    /// unmounted (i.e. it is `Mounted` or `Updating`).
    #[must_use]
    pub const fn is_mounted(self) -> bool {
        self.is_active()
    }

    /// Returns whether the instance has reached its terminal
    /// [`LifecycleState::Unmounted`] phase.
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
