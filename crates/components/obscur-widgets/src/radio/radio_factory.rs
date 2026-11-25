//! The registry entry that mints `radio` components (spec §2.2).

use obscur_identity::ComponentId;
use obscur_view::component::{
    ComponentContract, ComponentFactory, ComponentInstance, ErasedComponent, InstanceId,
};

use super::{Radio, RadioComponent, RadioProps};

#[doc = include_str!("../../docs/api/radio/radio_factory.doc/RadioFactory.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RadioFactory;

impl ComponentFactory for RadioFactory {
    #[doc = include_str!("../../docs/api/radio/radio_factory.doc/component_id.md")]
    fn component_id(&self) -> ComponentId {
        Radio::COMPONENT_ID
    }

    #[doc = include_str!("../../docs/api/radio/radio_factory.doc/contract.md")]
    fn contract(&self) -> ComponentContract {
        RadioProps::contract()
    }

    /// A fresh instance handle for one mounted `<radio>`.
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, Radio::COMPONENT_ID)
    }

    /// A fresh [`RadioComponent`](super::RadioComponent) behind the erased handle the
    /// registry stores.
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(RadioComponent)
    }
}

#[cfg(test)]
mod tests {
    use super::RadioFactory;
    use crate::radio::Radio;
    use obscur_view::component::{ComponentFactory, InstanceId};

    #[test]
    fn mounts_under_the_id_the_tag_and_the_builder_share() {
        // The one thing a factory can get wrong that nothing else would catch: an
        // id no authored node carries mounts on nothing, silently.
        let factory = RadioFactory;
        assert_eq!(factory.component_id(), Radio::COMPONENT_ID);
        assert_eq!(factory.contract().id, Radio::COMPONENT_ID);
        assert_eq!(
            factory
                .create_instance(InstanceId::from_raw(1))
                .component_id(),
            Radio::COMPONENT_ID
        );
    }

    #[test]
    fn is_object_safe_behind_the_registry_handle() {
        let boxed: Box<dyn ComponentFactory> = Box::new(RadioFactory);
        assert_eq!(boxed.component_id(), Radio::COMPONENT_ID);
        let _erased = boxed.create_component();
    }
}
