//! `ComponentInstance`: the runtime record of one mounted component.

use obscur_identity::ComponentId;

use crate::component::InstanceId;
use crate::lifecycle::LifecycleState;
use crate::state::SubscriptionScope;

/// The runtime-side record of a single mounted component instance (spec §2.1).
///
/// This is *plain runtime data*, not the erased component object itself
/// (holding and dispatching the author's `Component` value is the registry's
/// job in a later sub-step). An instance records its own [`InstanceId`], the
/// [`ComponentId`] kind it instantiates, its current [`LifecycleState`], and its
/// owned [`SubscriptionScope`].
///
/// The lifecycle-advancing methods move the state through
/// `Created → Mounted → (Updating ⇄ Mounted)* → Unmounted`. They are
/// panic-free: an invalid transition is a no-op that returns `false` rather than
/// aborting. On [`unmount`](Self::unmount) the subscription scope is cleared,
/// modelling the spec's "automatic cleanup".
#[derive(Debug)]
pub struct ComponentInstance {
    instance: InstanceId,
    component: ComponentId,
    state: LifecycleState,
    scope: SubscriptionScope,
}

impl ComponentInstance {
    /// Creates a freshly constructed instance in [`LifecycleState::Created`]
    /// with an empty subscription scope.
    #[must_use]
    pub fn new(instance: InstanceId, component: ComponentId) -> Self {
        Self {
            instance,
            component,
            state: LifecycleState::Created,
            scope: SubscriptionScope::new(),
        }
    }

    /// This instance's identity.
    #[must_use]
    pub const fn id(&self) -> InstanceId {
        self.instance
    }

    /// The component kind this is an instance of.
    #[must_use]
    pub const fn component_id(&self) -> ComponentId {
        self.component
    }

    /// The instance's current lifecycle phase.
    #[must_use]
    pub const fn state(&self) -> LifecycleState {
        self.state
    }

    /// Shared access to the instance's scoped subscriptions.
    #[must_use]
    pub const fn scope(&self) -> &SubscriptionScope {
        &self.scope
    }

    /// Mutable access to the instance's scoped subscriptions.
    #[must_use]
    pub fn scope_mut(&mut self) -> &mut SubscriptionScope {
        &mut self.scope
    }

    /// Advances a `Created` instance to [`LifecycleState::Mounted`].
    ///
    /// Returns `true` if the transition applied; `false` if the instance was
    /// not in [`LifecycleState::Created`].
    pub fn mount(&mut self) -> bool {
        if matches!(self.state, LifecycleState::Created) {
            self.state = LifecycleState::Mounted;
            true
        } else {
            false
        }
    }

    /// Moves a `Mounted` instance into [`LifecycleState::Updating`].
    ///
    /// Returns `true` if the transition applied; `false` otherwise.
    pub fn begin_update(&mut self) -> bool {
        if matches!(self.state, LifecycleState::Mounted) {
            self.state = LifecycleState::Updating;
            true
        } else {
            false
        }
    }

    /// Settles an `Updating` instance back to [`LifecycleState::Mounted`].
    ///
    /// Returns `true` if the transition applied; `false` otherwise.
    pub fn end_update(&mut self) -> bool {
        if matches!(self.state, LifecycleState::Updating) {
            self.state = LifecycleState::Mounted;
            true
        } else {
            false
        }
    }

    /// Tears the instance down to [`LifecycleState::Unmounted`] and clears its
    /// subscription scope (automatic cleanup, spec §2.1).
    ///
    /// Valid from any active phase (`Mounted` or `Updating`). Returns `true` if
    /// the transition applied; an already-unmounted or never-mounted instance
    /// returns `false` and is left untouched.
    pub fn unmount(&mut self) -> bool {
        if self.state.is_active() {
            self.scope.clear();
            self.state = LifecycleState::Unmounted;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ComponentInstance;
    use crate::component::InstanceId;
    use crate::lifecycle::LifecycleState;
    use crate::state::StateGeneration;
    use obscur_identity::ComponentId;
    use obscur_identity::SignalId;

    fn instance() -> ComponentInstance {
        ComponentInstance::new(InstanceId::from_raw(1), ComponentId::from_raw(7))
    }

    #[test]
    fn starts_created() {
        let inst = instance();
        assert_eq!(inst.state(), LifecycleState::Created);
        assert_eq!(inst.id(), InstanceId::from_raw(1));
        assert_eq!(inst.component_id(), ComponentId::from_raw(7));
    }

    #[test]
    fn full_lifecycle_transitions() {
        let mut inst = instance();
        assert!(inst.mount());
        assert_eq!(inst.state(), LifecycleState::Mounted);
        assert!(inst.begin_update());
        assert_eq!(inst.state(), LifecycleState::Updating);
        assert!(inst.end_update());
        assert_eq!(inst.state(), LifecycleState::Mounted);
        assert!(inst.unmount());
        assert_eq!(inst.state(), LifecycleState::Unmounted);
    }

    #[test]
    fn invalid_transitions_are_noops() {
        let mut inst = instance();
        // Cannot update before mounting.
        assert!(!inst.begin_update());
        assert!(!inst.end_update());
        // Cannot mount twice.
        assert!(inst.mount());
        assert!(!inst.mount());
        // Cannot unmount twice.
        assert!(inst.unmount());
        assert!(!inst.unmount());
    }

    #[test]
    fn unmount_clears_subscriptions() {
        let mut inst = instance();
        inst.mount();
        inst.scope_mut()
            .subscribe(SignalId::from_raw(1), StateGeneration::INITIAL);
        assert_eq!(inst.scope().len(), 1);
        inst.unmount();
        assert!(inst.scope().is_empty());
    }
}
