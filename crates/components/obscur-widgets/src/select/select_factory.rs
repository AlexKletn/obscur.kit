#![doc = include_str!("select_factory.doc/_module.md")]

use obscur_identity::ComponentId;
use obscur_view::component::{
    ComponentContract, ComponentFactory, ComponentInstance, ErasedComponent, InstanceId,
};

use super::{Select, SelectComponent, SelectProps};

#[doc = include_str!("select_factory.doc/SelectFactory.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SelectFactory;

impl ComponentFactory for SelectFactory {
    #[doc = include_str!("select_factory.doc/component_id.md")]
    fn component_id(&self) -> ComponentId {
        Select::COMPONENT_ID
    }

    #[doc = include_str!("select_factory.doc/contract.md")]
    fn contract(&self) -> ComponentContract {
        SelectProps::contract()
    }

    #[doc = include_str!("select_factory.doc/create_instance.md")]
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, Select::COMPONENT_ID)
    }

    #[doc = include_str!("select_factory.doc/create_component.md")]
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(SelectComponent)
    }
}

#[cfg(test)]
mod tests {
    use super::SelectFactory;
    use crate::select::Select;
    use obscur_view::component::{ComponentFactory, InstanceId};

    #[test]
    fn mounts_under_the_id_the_tag_and_the_builder_share() {
        // The one thing a factory can get wrong that nothing else would catch: an
        // id no authored node carries mounts on nothing, silently.
        let factory = SelectFactory;
        assert_eq!(factory.component_id(), Select::COMPONENT_ID);
        assert_eq!(factory.contract().id, Select::COMPONENT_ID);
        assert_eq!(
            factory
                .create_instance(InstanceId::from_raw(1))
                .component_id(),
            Select::COMPONENT_ID
        );
    }

    #[test]
    fn is_object_safe_behind_the_registry_handle() {
        let boxed: Box<dyn ComponentFactory> = Box::new(SelectFactory);
        assert_eq!(boxed.component_id(), Select::COMPONENT_ID);
        let _erased = boxed.create_component();
    }
}
