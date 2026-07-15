//! `ComponentRegistration`: one entry in the [`ComponentRegistry`](crate::component::ComponentRegistry).

use obscur_identity::ComponentId;

use crate::component::{
    ComponentContract, ComponentFactory, ComponentInstance, ComponentVersion, ErasedComponent,
    InstanceId,
};

/// One entry in the component registry (spec §2.14).
///
/// It aggregates exactly what §2.14 lists a registry stores, without
/// duplication: the stable [`ComponentId`], the [`ComponentContract`] — which
/// already carries the contract schema, the component's capability requirements,
/// and its diagnostics metadata (spec §2.13) — the [`ComponentVersion`] of that
/// schema, and the boxed [`ComponentFactory`] used to instantiate the kind.
///
/// # `Debug` and the boxed factory
///
/// This type owns a trait object (`Box<dyn ComponentFactory>`), so it cannot
/// derive `Clone`, `PartialEq`, or `Eq`. It *does* derive [`Debug`]: that is why
/// [`ComponentFactory`] carries a [`core::fmt::Debug`] supertrait — the boxed
/// factory is `Debug`, so no hand-written impl is needed.
#[derive(Debug)]
pub struct ComponentRegistration {
    id: ComponentId,
    contract: ComponentContract,
    version: ComponentVersion,
    factory: Box<dyn ComponentFactory>,
}

impl ComponentRegistration {
    /// Builds a registration for `factory` at `version`.
    ///
    /// The [`ComponentId`] and [`ComponentContract`] are taken from the factory
    /// itself (via [`ComponentFactory::component_id`] and
    /// [`ComponentFactory::contract`]), keeping the entry internally consistent
    /// by construction.
    #[must_use]
    pub fn new(factory: Box<dyn ComponentFactory>, version: ComponentVersion) -> Self {
        Self {
            id: factory.component_id(),
            contract: factory.contract(),
            version,
            factory,
        }
    }

    /// The stable identity of the registered component kind.
    #[must_use]
    pub fn id(&self) -> ComponentId {
        self.id
    }

    /// The registered component's contract (schema, capability requirements, and
    /// diagnostics metadata all live here — spec §2.13).
    #[must_use]
    pub fn contract(&self) -> &ComponentContract {
        &self.contract
    }

    /// The version of this component's contract schema.
    #[must_use]
    pub fn version(&self) -> ComponentVersion {
        self.version
    }

    /// The factory used to instantiate this component kind.
    #[must_use]
    pub fn factory(&self) -> &dyn ComponentFactory {
        self.factory.as_ref()
    }

    /// Mints a fresh runtime instance of this kind via the stored factory.
    ///
    /// The returned instance is in
    /// [`LifecycleState::Created`](crate::lifecycle::LifecycleState::Created).
    #[must_use]
    pub fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        self.factory.create_instance(instance)
    }

    /// Mints a fresh renderable component value of this kind via the stored
    /// factory — the [`create_instance`](Self::create_instance) counterpart that
    /// carries the component's local state (spec §2.1).
    #[must_use]
    pub fn create_component(&self) -> Box<dyn ErasedComponent> {
        self.factory.create_component()
    }
}

#[cfg(test)]
mod tests {
    use super::ComponentRegistration;
    use crate::component::{
        Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
        ComponentVersion, ErasedComponent, FromProps, InstanceId, PropsError,
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
            ComponentVersion::new(1, 2),
        )
    }

    #[test]
    fn derives_id_and_contract_from_factory() {
        let reg = registration(5);
        assert_eq!(reg.id(), ComponentId::from_raw(5));
        assert_eq!(reg.contract().id, ComponentId::from_raw(5));
        assert_eq!(reg.version(), ComponentVersion::new(1, 2));
        assert_eq!(reg.factory().component_id(), ComponentId::from_raw(5));
    }

    #[test]
    fn mints_created_instance() {
        let reg = registration(5);
        let inst = reg.create_instance(InstanceId::from_raw(9));
        assert_eq!(inst.id(), InstanceId::from_raw(9));
        assert_eq!(inst.component_id(), ComponentId::from_raw(5));
    }

    #[test]
    fn is_debug() {
        // Proves the boxed factory's Debug supertrait makes the entry Debug.
        let _ = format!("{:?}", registration(5));
    }
}
