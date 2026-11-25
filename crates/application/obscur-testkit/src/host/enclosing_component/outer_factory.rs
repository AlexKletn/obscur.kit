use obscur_identity::ComponentId;
use obscur_view::component::{
    ComponentContract, ComponentFactory, ComponentInstance, ErasedComponent, InstanceId,
};

use super::{EnclosingComponent, outer::Outer};

/// Mints the one [`Outer`]; it carries no configuration.
#[derive(Debug)]
pub(super) struct OuterFactory;

impl ComponentFactory for OuterFactory {
    fn component_id(&self) -> ComponentId {
        EnclosingComponent::ID
    }
    fn contract(&self) -> ComponentContract {
        ComponentContract::new(EnclosingComponent::ID)
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, EnclosingComponent::ID)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Outer)
    }
}
