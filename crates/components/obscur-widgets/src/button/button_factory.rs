#![doc = include_str!("button_factory.doc/_module.md")]

use obscur_identity::ComponentId;
use obscur_view::component::{
    ComponentContract, ComponentFactory, ComponentInstance, ErasedComponent, InstanceId,
};

use super::{Button, ButtonComponent, ButtonProps};

#[doc = include_str!("button_factory.doc/ButtonFactory.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ButtonFactory;

impl ComponentFactory for ButtonFactory {
    #[doc = include_str!("button_factory.doc/component_id.md")]
    fn component_id(&self) -> ComponentId {
        Button::COMPONENT_ID
    }

    #[doc = include_str!("button_factory.doc/contract.md")]
    fn contract(&self) -> ComponentContract {
        ButtonProps::contract()
    }

    #[doc = include_str!("button_factory.doc/create_instance.md")]
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, Button::COMPONENT_ID)
    }

    #[doc = include_str!("button_factory.doc/create_component.md")]
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(ButtonComponent)
    }
}

#[cfg(test)]
mod tests {
    use super::ButtonFactory;
    use crate::button::Button;
    use obscur_view::component::{ComponentFactory, InstanceId};

    #[test]
    fn mounts_under_the_id_the_tag_and_the_builder_share() {
        // The one thing a factory can get wrong that nothing else would catch: an
        // id no authored node carries mounts on nothing, silently.
        let factory = ButtonFactory;
        assert_eq!(factory.component_id(), Button::COMPONENT_ID);
        assert_eq!(factory.contract().id, Button::COMPONENT_ID);
        assert_eq!(
            factory
                .create_instance(InstanceId::from_raw(1))
                .component_id(),
            Button::COMPONENT_ID
        );
    }

    #[test]
    fn is_object_safe_behind_the_registry_handle() {
        let boxed: Box<dyn ComponentFactory> = Box::new(ButtonFactory);
        assert_eq!(boxed.component_id(), Button::COMPONENT_ID);
        let _erased = boxed.create_component();
    }
}
