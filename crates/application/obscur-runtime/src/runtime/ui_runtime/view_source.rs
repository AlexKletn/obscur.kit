#![doc = include_str!("view_source.doc/_module.md")]

use obscur_view::component::{ComponentRegistration, RegistryError};
use obscur_view::view::View;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("view_source.doc/set_root_view.md")]
    pub fn set_root_view(&mut self, builder: impl FnMut() -> View + 'static) {
        self.root_view = Some(Box::new(builder));
    }

    #[doc = include_str!("view_source.doc/has_root_view.md")]
    #[must_use]
    pub fn has_root_view(&self) -> bool {
        self.root_view.is_some()
    }

    #[doc = include_str!("view_source.doc/register_component.md")]
    pub fn register_component(
        &mut self,
        registration: ComponentRegistration,
    ) -> Result<(), RegistryError> {
        self.components.register(registration)
    }

    #[doc = include_str!("view_source.doc/mounted_component_count.md")]
    #[must_use]
    pub fn mounted_component_count(&self) -> usize {
        self.components.len()
    }
}
