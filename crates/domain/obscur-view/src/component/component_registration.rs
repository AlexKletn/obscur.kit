#![doc = include_str!("component_registration.doc/_module.md")]

use obscur_identity::ComponentId;

use crate::component::{
    ComponentContract, ComponentFactory, ComponentInstance, ComponentVersion, ErasedComponent,
    InstanceId,
};

#[doc = include_str!("component_registration.doc/ComponentRegistration.md")]
#[derive(Debug)]
pub struct ComponentRegistration {
    id: ComponentId,
    contract: ComponentContract,
    version: ComponentVersion,
    factory: Box<dyn ComponentFactory>,
}

impl ComponentRegistration {
    #[doc = include_str!("component_registration.doc/new.md")]
    #[must_use]
    pub fn new(factory: Box<dyn ComponentFactory>, version: ComponentVersion) -> Self {
        Self {
            id: factory.component_id(),
            contract: factory.contract(),
            version,
            factory,
        }
    }

    #[doc = include_str!("component_registration.doc/id.md")]
    #[must_use]
    pub fn id(&self) -> ComponentId {
        self.id
    }

    #[doc = include_str!("component_registration.doc/contract.md")]
    #[must_use]
    pub fn contract(&self) -> &ComponentContract {
        &self.contract
    }

    #[doc = include_str!("component_registration.doc/version.md")]
    #[must_use]
    pub fn version(&self) -> ComponentVersion {
        self.version
    }

    #[doc = include_str!("component_registration.doc/factory.md")]
    #[must_use]
    pub fn factory(&self) -> &dyn ComponentFactory {
        self.factory.as_ref()
    }

    #[doc = include_str!("component_registration.doc/create_instance.md")]
    #[must_use]
    pub fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        self.factory.create_instance(instance)
    }

    #[doc = include_str!("component_registration.doc/create_component.md")]
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
