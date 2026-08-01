#![doc = include_str!("component_instance.doc/_module.md")]

use obscur_identity::ComponentId;

use crate::component::InstanceId;
use crate::lifecycle::LifecycleState;
use crate::state::SubscriptionScope;

#[doc = include_str!("component_instance.doc/ComponentInstance.md")]
#[derive(Debug)]
pub struct ComponentInstance {
    instance: InstanceId,
    component: ComponentId,
    state: LifecycleState,
    scope: SubscriptionScope,
}

impl ComponentInstance {
    #[doc = include_str!("component_instance.doc/new.md")]
    #[must_use]
    pub fn new(instance: InstanceId, component: ComponentId) -> Self {
        Self {
            instance,
            component,
            state: LifecycleState::Created,
            scope: SubscriptionScope::new(),
        }
    }

    #[doc = include_str!("component_instance.doc/id.md")]
    #[must_use]
    pub const fn id(&self) -> InstanceId {
        self.instance
    }

    #[doc = include_str!("component_instance.doc/component_id.md")]
    #[must_use]
    pub const fn component_id(&self) -> ComponentId {
        self.component
    }

    #[doc = include_str!("component_instance.doc/state.md")]
    #[must_use]
    pub const fn state(&self) -> LifecycleState {
        self.state
    }

    #[doc = include_str!("component_instance.doc/scope.md")]
    #[must_use]
    pub const fn scope(&self) -> &SubscriptionScope {
        &self.scope
    }

    #[doc = include_str!("component_instance.doc/scope_mut.md")]
    #[must_use]
    pub fn scope_mut(&mut self) -> &mut SubscriptionScope {
        &mut self.scope
    }

    #[doc = include_str!("component_instance.doc/mount.md")]
    pub fn mount(&mut self) -> bool {
        if matches!(self.state, LifecycleState::Created) {
            self.state = LifecycleState::Mounted;
            true
        } else {
            false
        }
    }

    #[doc = include_str!("component_instance.doc/begin_update.md")]
    pub fn begin_update(&mut self) -> bool {
        if matches!(self.state, LifecycleState::Mounted) {
            self.state = LifecycleState::Updating;
            true
        } else {
            false
        }
    }

    #[doc = include_str!("component_instance.doc/end_update.md")]
    pub fn end_update(&mut self) -> bool {
        if matches!(self.state, LifecycleState::Updating) {
            self.state = LifecycleState::Mounted;
            true
        } else {
            false
        }
    }

    #[doc = include_str!("component_instance.doc/unmount.md")]
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
