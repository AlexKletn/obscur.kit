//! The set of shipped controls a runtime mounts by default (spec §2.2).

use crate::view::component::{ComponentRegistration, ComponentVersion, RegistryError};
use crate::widgets::button::ButtonFactory;
use crate::widgets::checkbox::CheckboxFactory;
use crate::widgets::input::InputFactory;
use crate::widgets::radio::RadioFactory;
use crate::widgets::select::SelectFactory;

#[doc = include_str!("../../docs/api/bootstrap/builtin_components.doc/BuiltinComponents.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuiltinComponents;

impl BuiltinComponents {
    /// The version every built-in registers under (§2.2). `1.0`: the shipped
    /// controls have one shape, and a host that needs a different one replaces the
    /// registration rather than negotiating a minor version with it.
    const VERSION: ComponentVersion = ComponentVersion::new(1, 0);

    #[doc = include_str!("../../docs/api/bootstrap/builtin_components.doc/registrations.md")]
    #[must_use]
    pub fn registrations() -> Vec<ComponentRegistration> {
        vec![
            ComponentRegistration::new(Box::new(CheckboxFactory), Self::VERSION),
            ComponentRegistration::new(Box::new(RadioFactory), Self::VERSION),
            ComponentRegistration::new(Box::new(SelectFactory), Self::VERSION),
            ComponentRegistration::new(Box::new(InputFactory), Self::VERSION),
            ComponentRegistration::new(Box::new(ButtonFactory), Self::VERSION),
        ]
    }

    #[doc = include_str!("../../docs/api/bootstrap/builtin_components.doc/install.md")]
    pub fn install(runtime: &mut crate::runtime::UiRuntime) -> Result<(), RegistryError> {
        for registration in Self::registrations() {
            runtime.register_component(registration)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BuiltinComponents;
    use crate::geometry::Size;
    use crate::identity::RuntimeId;
    use crate::runtime::UiRuntime;
    use crate::view::component::{ComponentRegistration, RegistryError};
    use crate::widgets::button::Button;
    use crate::widgets::checkbox::Checkbox;
    use crate::widgets::input::Input;
    use crate::widgets::radio::Radio;
    use crate::widgets::select::Select;

    fn runtime() -> UiRuntime {
        UiRuntime::new(RuntimeId::from_raw(1), Size::new(320.0, 200.0))
    }

    #[test]
    fn every_registration_claims_a_shipped_widgets_id() {
        // A registration under an id no authored node carries would mount on
        // nothing, silently — the exact failure this whole path exists to end.
        let ids: Vec<_> = BuiltinComponents::registrations()
            .iter()
            .map(ComponentRegistration::id)
            .collect();
        assert_eq!(
            ids,
            vec![
                Checkbox::COMPONENT_ID,
                Radio::COMPONENT_ID,
                Select::COMPONENT_ID,
                Input::COMPONENT_ID,
                Button::COMPONENT_ID
            ]
        );
    }

    #[test]
    fn the_table_id_is_left_unclaimed() {
        // `<table>` mounts nothing: its rows and cells are `tr`/`td`/`th` the
        // author writes or the Rust builder generates, so a component there
        // could only hand them straight back. Stated as an assertion so that
        // adding a `TableFactory` is a deliberate act with an answer to "whose
        // content are the rows?" behind it.
        let table = crate::widgets::table::Table::COMPONENT_ID;
        assert!(
            !BuiltinComponents::registrations()
                .iter()
                .any(|registration| registration.id() == table)
        );
    }

    #[test]
    fn installing_claims_the_ids_and_a_second_install_reports_the_clash() {
        // The registry is not readable from outside, so the proof that the first
        // install took is that the second one is refused — under the built-in's
        // own id, which is also the proof that nothing shadowed anything.
        let mut runtime = runtime();
        BuiltinComponents::install(&mut runtime).expect("the built-ins register");
        assert_eq!(
            BuiltinComponents::install(&mut runtime),
            Err(RegistryError::DuplicateComponent(Checkbox::COMPONENT_ID))
        );

        // …and that is only the FIRST id, so it says nothing about the ones behind
        // it. Ask about each separately: a registration that never reached the
        // registry would be accepted here rather than refused.
        for registration in BuiltinComponents::registrations() {
            let id = registration.id();
            assert_eq!(
                runtime.register_component(registration),
                Err(RegistryError::DuplicateComponent(id)),
                "{id:?} must already be claimed"
            );
        }
    }
}
