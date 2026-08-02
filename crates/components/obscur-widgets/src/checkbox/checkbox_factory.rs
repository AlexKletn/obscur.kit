#![doc = include_str!("checkbox_factory.doc/_module.md")]

use obscur_identity::ComponentId;
use obscur_view::component::{
    ComponentContract, ComponentFactory, ComponentInstance, ErasedComponent, InstanceId,
};

use super::{Checkbox, CheckboxComponent, CheckboxProps};

#[doc = include_str!("checkbox_factory.doc/CheckboxFactory.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CheckboxFactory;

impl ComponentFactory for CheckboxFactory {
    #[doc = include_str!("checkbox_factory.doc/component_id.md")]
    fn component_id(&self) -> ComponentId {
        Checkbox::COMPONENT_ID
    }

    #[doc = include_str!("checkbox_factory.doc/contract.md")]
    fn contract(&self) -> ComponentContract {
        CheckboxProps::contract()
    }

    #[doc = include_str!("checkbox_factory.doc/create_instance.md")]
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, Checkbox::COMPONENT_ID)
    }

    #[doc = include_str!("checkbox_factory.doc/create_component.md")]
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(CheckboxComponent)
    }
}

#[cfg(test)]
mod tests {
    use super::CheckboxFactory;
    use crate::checkbox::Checkbox;
    use obscur_view::component::{ComponentFactory, InstanceId};

    #[test]
    fn mounts_under_the_id_the_tag_and_the_builder_share() {
        // The one thing a factory can get wrong that nothing else would catch: an
        // id no authored node carries mounts on nothing, silently.
        let factory = CheckboxFactory;
        assert_eq!(factory.component_id(), Checkbox::COMPONENT_ID);
        assert_eq!(factory.contract().id, Checkbox::COMPONENT_ID);
        assert_eq!(
            factory
                .create_instance(InstanceId::from_raw(1))
                .component_id(),
            Checkbox::COMPONENT_ID
        );
    }

    #[test]
    fn is_object_safe_behind_the_registry_handle() {
        let boxed: Box<dyn ComponentFactory> = Box::new(CheckboxFactory);
        assert_eq!(boxed.component_id(), Checkbox::COMPONENT_ID);
        let _erased = boxed.create_component();
    }
}
