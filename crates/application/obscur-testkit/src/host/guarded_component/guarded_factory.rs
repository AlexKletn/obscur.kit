use obscur_identity::ComponentId;
use obscur_view::boundary::ErrorBoundaryPolicy;
use obscur_view::component::{
    ComponentContract, ComponentFactory, ComponentInstance, ErasedComponent, InstanceId,
};
use obscur_view::view::View;

use super::{GuardedComponent, guarded::Guarded};

/// Mints [`Guarded`]s carrying the caller's policy and fallback.
#[derive(Debug)]
pub(super) struct GuardedFactory {
    pub(super) policy: ErrorBoundaryPolicy,
    pub(super) fallback: Option<View>,
}

impl ComponentFactory for GuardedFactory {
    fn component_id(&self) -> ComponentId {
        GuardedComponent::ID
    }
    fn contract(&self) -> ComponentContract {
        ComponentContract::new(GuardedComponent::ID)
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, GuardedComponent::ID)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Guarded {
            policy: self.policy,
            fallback: self.fallback.clone(),
        })
    }
}
