//! `ComponentFactory`: the object-safe type-erasure seam for instantiating one
//! component kind (spec §2.14).

use obscur_identity::ComponentId;

use crate::component::{ComponentContract, ComponentInstance, ErasedComponent, InstanceId};

#[doc = include_str!("../../docs/api/component/component_factory.doc/ComponentFactory.md")]
pub trait ComponentFactory: core::fmt::Debug {
    #[doc = include_str!("../../docs/api/component/component_factory.doc/component_id.md")]
    fn component_id(&self) -> ComponentId;

    #[doc = include_str!("../../docs/api/component/component_factory.doc/contract.md")]
    fn contract(&self) -> ComponentContract;

    #[doc = include_str!("../../docs/api/component/component_factory.doc/create_instance.md")]
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance;

    #[doc = include_str!("../../docs/api/component/component_factory.doc/create_component.md")]
    fn create_component(&self) -> Box<dyn ErasedComponent>;
}

#[cfg(test)]
mod tests {
    use super::ComponentFactory;
    use crate::component::{
        Component, ComponentContext, ComponentContract, ComponentInstance, ErasedComponent,
        FromProps, InstanceId, PropsError,
    };
    use crate::view::{View, ViewProperty};
    use obscur_identity::ComponentId;

    /// A trivial factory whose contract is the empty-but-valid contract for its
    /// id, used to prove the trait is object-safe and honest.
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

    #[test]
    fn is_object_safe_and_mints_created_instances() {
        let factory: Box<dyn ComponentFactory> = Box::new(StubFactory {
            id: ComponentId::from_raw(3),
        });

        assert_eq!(factory.component_id(), ComponentId::from_raw(3));
        assert_eq!(factory.contract().id, ComponentId::from_raw(3));

        let instance = factory.create_instance(InstanceId::from_raw(1));
        assert_eq!(instance.id(), InstanceId::from_raw(1));
        assert_eq!(instance.component_id(), ComponentId::from_raw(3));
        assert_eq!(instance.state(), crate::lifecycle::LifecycleState::Created);
    }
}
