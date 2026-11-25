use obscur_identity::{ComponentId, SignalId};
use obscur_view::component::{
    ComponentContract, ComponentFactory, ComponentInstance, ErasedComponent, InstanceId,
};
use obscur_view::state::LocalState;

use super::{CounterComponent, counter::Counter};

/// Mints [`Counter`]s carrying the caller's self-update budget and tracked
/// source.
#[derive(Debug)]
pub(super) struct CounterFactory {
    pub(super) self_updates: u32,
    pub(super) tracks: Option<SignalId>,
}

impl ComponentFactory for CounterFactory {
    fn component_id(&self) -> ComponentId {
        CounterComponent::ID
    }
    fn contract(&self) -> ComponentContract {
        ComponentContract::new(CounterComponent::ID)
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, CounterComponent::ID)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Counter {
            total: LocalState::new(0),
            self_updates: self.self_updates,
            tracks: self.tracks,
        })
    }
}
