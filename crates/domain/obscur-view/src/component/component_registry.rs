//! `ComponentRegistry`: the static-default store of component registrations
//! (spec §2.14).

use std::collections::{HashMap, HashSet};

use obscur_capability::CapabilityId;
use obscur_identity::ComponentId;

use crate::component::{
    ComponentContract, ComponentInstance, ComponentRegistration, ErasedComponent, InstanceId,
    RegistryError,
};
use crate::contract::CapabilityRequirement;

#[doc = include_str!("../../docs/api/component/component_registry.doc/ComponentRegistry.md")]
#[derive(Debug, Default)]
pub struct ComponentRegistry {
    entries: HashMap<ComponentId, ComponentRegistration>,
    granted: HashSet<CapabilityId>,
}

impl ComponentRegistry {
    /// Creates an empty registry with **no capabilities granted**.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            granted: HashSet::new(),
        }
    }

    #[doc = include_str!("../../docs/api/component/component_registry.doc/grant.md")]
    pub fn grant(&mut self, capability: CapabilityId) {
        self.granted.insert(capability);
    }

    /// Returns whether `capability` has been granted on this registry.
    #[must_use]
    pub fn is_granted(&self, capability: &CapabilityId) -> bool {
        self.granted.contains(capability)
    }

    /// Returns whether every [`CapabilityRequirement::Required`] capability in
    /// `contract` has been granted — the gate [`create_instance`](Self::create_instance)
    /// applies before minting an instance.
    #[must_use]
    fn required_capabilities_satisfied(&self, contract: &ComponentContract) -> bool {
        contract
            .capabilities
            .iter()
            .filter(|c| c.requirement == CapabilityRequirement::Required)
            .all(|c| self.granted.contains(&c.capability))
    }

    #[doc = include_str!("../../docs/api/component/component_registry.doc/register.md")]
    pub fn register(&mut self, registration: ComponentRegistration) -> Result<(), RegistryError> {
        let id = registration.id();
        if self.entries.contains_key(&id) {
            return Err(RegistryError::DuplicateComponent(id));
        }
        self.entries.insert(id, registration);
        Ok(())
    }

    /// Returns the registration for `id`, if any.
    #[must_use]
    pub fn get(&self, id: ComponentId) -> Option<&ComponentRegistration> {
        self.entries.get(&id)
    }

    /// Returns the contract for the component registered under `id`, if any.
    #[must_use]
    pub fn contract(&self, id: ComponentId) -> Option<&ComponentContract> {
        self.entries.get(&id).map(ComponentRegistration::contract)
    }

    #[doc = include_str!("../../docs/api/component/component_registry.doc/create_instance.md")]
    #[must_use]
    pub fn create_instance(
        &self,
        id: ComponentId,
        instance: InstanceId,
    ) -> Option<ComponentInstance> {
        let registration = self.entries.get(&id)?;
        if !self.required_capabilities_satisfied(registration.contract()) {
            return None;
        }
        Some(registration.create_instance(instance))
    }

    #[doc = include_str!("../../docs/api/component/component_registry.doc/create_component.md")]
    #[must_use]
    pub fn create_component(&self, id: ComponentId) -> Option<Box<dyn ErasedComponent>> {
        self.entries
            .get(&id)
            .map(ComponentRegistration::create_component)
    }

    /// Returns whether a component is registered under `id`.
    #[must_use]
    pub fn contains(&self, id: ComponentId) -> bool {
        self.entries.contains_key(&id)
    }

    /// The number of registered component kinds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the registry holds no registrations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::ComponentRegistry;
    use crate::component::{
        Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
        ComponentRegistration, ComponentVersion, ErasedComponent, FromProps, InstanceId,
        PropsError, RegistryError,
    };
    use crate::view::{View, ViewProperty};
    use obscur_identity::ComponentId;

    #[derive(Debug)]
    struct StubFactory {
        id: ComponentId,
    }

    /// A minimal real component, so the stub factory can mint a renderable
    /// value rather than pretend. Its props decode from anything.
    struct StubProps;
    impl FromProps for StubProps {
        fn from_props(_: &[ViewProperty]) -> Result<Self, PropsError> {
            Ok(Self)
        }
    }
    struct StubComponent;
    impl Component for StubComponent {
        type Props = StubProps;
        fn render(&mut self, _: &mut ComponentContext<'_>, _: &Self::Props) -> View {
            View::empty()
        }
    }

    impl ComponentFactory for StubFactory {
        fn component_id(&self) -> ComponentId {
            self.id
        }
        fn contract(&self) -> ComponentContract {
            ComponentContract::new(self.id)
        }
        fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
            ComponentInstance::new(instance, self.id)
        }

        fn create_component(&self) -> Box<dyn ErasedComponent> {
            Box::new(StubComponent)
        }
    }

    fn registration(raw: u64) -> ComponentRegistration {
        ComponentRegistration::new(
            Box::new(StubFactory {
                id: ComponentId::from_raw(raw),
            }),
            ComponentVersion::INITIAL,
        )
    }

    #[test]
    fn starts_empty() {
        let reg = ComponentRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
        assert!(!reg.contains(ComponentId::from_raw(1)));
        assert!(reg.get(ComponentId::from_raw(1)).is_none());
        assert!(reg.contract(ComponentId::from_raw(1)).is_none());
    }

    #[test]
    fn register_and_look_up() {
        let mut reg = ComponentRegistry::new();
        assert!(reg.register(registration(7)).is_ok());
        assert_eq!(reg.len(), 1);
        assert!(reg.contains(ComponentId::from_raw(7)));
        assert_eq!(
            reg.get(ComponentId::from_raw(7))
                .map(ComponentRegistration::id),
            Some(ComponentId::from_raw(7))
        );
        assert_eq!(
            reg.contract(ComponentId::from_raw(7)).map(|c| c.id),
            Some(ComponentId::from_raw(7))
        );
    }

    #[test]
    fn a_registered_component_renders_through_the_registry_by_id() {
        // The whole erasure chain, end to end and by ComponentId alone — which is
        // all a runtime has when it meets an `Element { component }` node:
        // registry → factory → erased component → decoded props → typed render.
        use crate::lifecycle::LifecycleState;
        use crate::state::{SignalGenerations, SubscriptionScope};

        let mut reg = ComponentRegistry::new();
        reg.register(registration(7)).expect("register");

        let mut component = reg
            .create_component(ComponentId::from_raw(7))
            .expect("a registered kind mints a component");

        let mut scope = SubscriptionScope::new();
        let mut needs_update = false;
        let signals = SignalGenerations::new();
        let mut ctx = ComponentContext::new(
            &mut scope,
            LifecycleState::Mounted,
            &mut needs_update,
            &signals,
        );
        let view = component
            .render_erased(&mut ctx, &[])
            .expect("the stub props decode from anything");
        assert!(view.root().is_empty(), "the stub renders an empty view");
    }

    #[test]
    fn an_unregistered_id_mints_nothing() {
        let reg = ComponentRegistry::new();
        assert!(reg.create_component(ComponentId::from_raw(7)).is_none());
        assert!(
            reg.create_instance(ComponentId::from_raw(7), InstanceId::from_raw(1))
                .is_none()
        );
    }

    #[test]
    fn duplicate_registration_is_rejected_without_panic() {
        let mut reg = ComponentRegistry::new();
        assert!(reg.register(registration(7)).is_ok());
        assert_eq!(
            reg.register(registration(7)),
            Err(RegistryError::DuplicateComponent(ComponentId::from_raw(7)))
        );
        // The original entry survives.
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn create_instance_uses_the_factory() {
        let mut reg = ComponentRegistry::new();
        reg.register(registration(7)).expect("first register");
        let inst = reg
            .create_instance(ComponentId::from_raw(7), InstanceId::from_raw(11))
            .expect("registered kind mints an instance");
        assert_eq!(inst.id(), InstanceId::from_raw(11));
        assert_eq!(inst.component_id(), ComponentId::from_raw(7));
    }

    #[test]
    fn create_instance_for_unregistered_id_is_none() {
        let reg = ComponentRegistry::new();
        assert!(
            reg.create_instance(ComponentId::from_raw(99), InstanceId::from_raw(1))
                .is_none()
        );
    }
}
