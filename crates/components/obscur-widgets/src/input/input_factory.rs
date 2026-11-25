//! The registry entry that mints text-entry components (spec §2.2).

use obscur_identity::ComponentId;
use obscur_view::component::{
    ComponentContract, ComponentFactory, ComponentInstance, ErasedComponent, InstanceId,
};

use super::{Input, InputComponent, InputProps};

#[doc = include_str!("../../docs/api/input/input_factory.doc/InputFactory.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InputFactory;

impl ComponentFactory for InputFactory {
    #[doc = include_str!("../../docs/api/input/input_factory.doc/component_id.md")]
    fn component_id(&self) -> ComponentId {
        Input::COMPONENT_ID
    }

    #[doc = include_str!("../../docs/api/input/input_factory.doc/contract.md")]
    fn contract(&self) -> ComponentContract {
        InputProps::contract()
    }

    /// A fresh instance handle for one mounted field.
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, Input::COMPONENT_ID)
    }

    /// A fresh [`InputComponent`](super::InputComponent) behind the erased handle
    /// the registry stores.
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(InputComponent)
    }
}

#[cfg(test)]
mod tests {
    use super::InputFactory;
    use crate::input::Input;
    use obscur_view::component::{ComponentFactory, InstanceId};

    #[test]
    fn mounts_under_the_id_both_text_entry_tags_share() {
        // The one thing a factory can get wrong that nothing else would catch: an
        // id no authored node carries mounts on nothing, silently. Here the id is
        // load-bearing twice over — `<input>` and `<textarea>` are one id, so a
        // single registration serves both tags.
        let factory = InputFactory;
        assert_eq!(factory.component_id(), Input::COMPONENT_ID);
        assert_eq!(factory.contract().id, Input::COMPONENT_ID);
        assert_eq!(
            factory
                .create_instance(InstanceId::from_raw(1))
                .component_id(),
            Input::COMPONENT_ID
        );
    }

    #[test]
    fn is_object_safe_behind_the_registry_handle() {
        let boxed: Box<dyn ComponentFactory> = Box::new(InputFactory);
        assert_eq!(boxed.component_id(), Input::COMPONENT_ID);
        let _erased = boxed.create_component();
    }
}
