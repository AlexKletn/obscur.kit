//! `ComponentRegistry`: the static-default store of component registrations
//! (spec §2.14).

use std::collections::HashMap;

use obscur_identity::ComponentId;

use crate::component::{
    ComponentContract, ComponentInstance, ComponentRegistration, ErasedComponent, InstanceId,
    RegistryError,
};

/// Maps each [`ComponentId`] to its [`ComponentRegistration`] (spec §2.14).
///
/// Per §2.14, **static registration is the default**, and this type *is* that
/// static-default store: it is a plain owned value that holds the component
/// kinds a runtime knows about. Dynamic registration (from packages/plugins) is
/// an extension the plugin system layers on top by calling [`register`] at
/// runtime; the registry itself imposes no such policy.
///
/// # No global singleton
///
/// A registry is an ordinary owned value — construct one with [`new`] or
/// [`Default`]. Multiple runtimes may each own their own registry; there is no
/// process-wide instance (see the architecture's "avoid global singletons").
///
/// # Erased render is deferred
///
/// [`create_instance`] mints the plain runtime [`ComponentInstance`] record in
/// [`LifecycleState::Created`](crate::lifecycle::LifecycleState::Created) via the
/// registered factory. It does **not** drive an erased render — decoding
/// `ViewProperty` inputs into each component's typed `Props` and invoking
/// `render` is a later runtime concern (see [`ComponentFactory`](crate::component::ComponentFactory)).
///
/// [`register`]: ComponentRegistry::register
/// [`new`]: ComponentRegistry::new
/// [`create_instance`]: ComponentRegistry::create_instance
#[derive(Debug, Default)]
pub struct ComponentRegistry {
    entries: HashMap<ComponentId, ComponentRegistration>,
}

impl ComponentRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Registers a component kind.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError::DuplicateComponent`] (carrying the offending
    /// [`ComponentId`]) if a registration with the same id already exists; the
    /// existing entry is left untouched. This never panics.
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

    /// Mints a fresh runtime instance of the component registered under `id`.
    ///
    /// Returns `None` if `id` is not registered. The minted
    /// [`ComponentInstance`] is in
    /// [`LifecycleState::Created`](crate::lifecycle::LifecycleState::Created).
    #[must_use]
    pub fn create_instance(
        &self,
        id: ComponentId,
        instance: InstanceId,
    ) -> Option<ComponentInstance> {
        self.entries
            .get(&id)
            .map(|registration| registration.create_instance(instance))
    }

    /// Mints a fresh renderable component value for `id`, or [`None`] when no
    /// component is registered under it.
    ///
    /// The [`create_instance`](Self::create_instance) counterpart: that returns
    /// the bookkeeping record, this returns the value the runtime renders through
    /// and which owns the component's local state.
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
