#![doc = include_str!("../../docs/api/input/input_props.doc/_module.md")]

use obscur_identity::InternedSymbol;
use obscur_view::component::{ComponentContract, FromProps, PropsError, prop_value};
use obscur_view::contract::PropContract;
use obscur_view::view::{ViewProperty, ViewValue};

use super::Input;

#[doc = include_str!("../../docs/api/input/input_props.doc/InputProps.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputProps {
    /// The field's text — what it displays, and what the editing session seeds from.
    pub value: String,
}

impl InputProps {
    #[doc = include_str!("../../docs/api/input/input_props.doc/VALUE.md")]
    pub const VALUE: InternedSymbol = InternedSymbol::from_raw(0xC25E_D3CA);

    /// Symbol of the component's name, `"Input"` — the FNV-1a-32 hash
    /// `#[derive(Contract)]` puts on `ComponentContract::with_name`.
    const NAME: InternedSymbol = InternedSymbol::from_raw(0x9F93_E11B);
    /// Opaque type identity of the `value` prop: the FNV-1a-32 hash of the Rust type
    /// spelling `String`, which is what `#[derive(Contract)]` records as a
    /// `PropContract`'s `value_type`. It names no view property and is compared,
    /// never interpreted.
    const VALUE_TYPE: InternedSymbol = InternedSymbol::from_raw(0x604F_4858);

    #[doc = include_str!("../../docs/api/input/input_props.doc/contract.md")]
    #[must_use]
    pub fn contract() -> ComponentContract {
        ComponentContract::new(Input::COMPONENT_ID)
            .with_name(Self::NAME)
            .with_prop(PropContract::new(Self::VALUE, Self::VALUE_TYPE).with_default())
    }
}

impl FromProps for InputProps {
    #[doc = include_str!("../../docs/api/input/input_props.doc/from_props.md")]
    fn from_props(props: &[ViewProperty]) -> Result<Self, PropsError> {
        let value = match prop_value(props, Input::PROP_VALUE) {
            Some(ViewValue::Text(text)) => text.to_string(),
            // No widget prop: this node came from markup, where the field's text
            // is the authored `value` attribute. A bare `<input/>` names no value
            // and is an empty — not a broken — field, so nothing here reports.
            _ => match prop_value(props, Self::VALUE) {
                Some(ViewValue::Text(authored)) => authored.to_string(),
                _ => String::new(),
            },
        };
        Ok(Self { value })
    }
}

#[cfg(test)]
mod tests {
    use super::InputProps;
    use crate::input::Input;
    use obscur_identity::InternedSymbol;
    use obscur_view::component::FromProps;
    use obscur_view::view::{PLACEHOLDER, READONLY, ViewNode, ViewProperty, ViewValue};

    fn decoded(props: &[ViewProperty]) -> InputProps {
        InputProps::from_props(props).expect("an input decodes without a required prop")
    }

    #[test]
    fn a_bare_input_decodes_to_an_empty_field() {
        // Every prop is defaulted: `<input/>` is a legal, empty field, so nothing
        // here may report. (A checkbox reports a missing label; a field has no
        // equivalent, because an empty field is the ordinary starting state.)
        assert_eq!(decoded(&[]).value, "");
    }

    #[test]
    fn an_authored_value_attribute_is_the_fields_text() {
        let props = [ViewProperty::new(InputProps::VALUE, ViewValue::from("abc"))];
        assert_eq!(decoded(&props).value, "abc");
    }

    #[test]
    fn the_widgets_own_prop_wins_over_the_authored_attribute() {
        // The Rust-API path. `Input::view` states the buffer text exactly, and it
        // is emitted on every node the builder makes — including the empty ones,
        // where reading the authored spelling instead would let a stale attribute
        // resurrect text the buffer no longer holds.
        let props = [
            ViewProperty::new(Input::PROP_VALUE, ViewValue::from("")),
            ViewProperty::new(InputProps::VALUE, ViewValue::from("stale")),
        ];
        assert_eq!(decoded(&props).value, "");
    }

    #[test]
    fn decodes_the_node_the_rust_builder_produces() {
        // The idempotence precondition: everything this decoder needs is on the
        // node `Input::view` builds, so mounting the component on one changes
        // nothing.
        let view = Input::with_value(InternedSymbol::from_raw(0x5678), "hello").view();
        let ViewNode::Element { props, .. } = &view.root().nodes()[0] else {
            panic!("expected element root");
        };
        assert_eq!(
            InputProps::from_props(props),
            Ok(InputProps {
                value: "hello".into()
            })
        );
    }

    #[test]
    fn the_placeholder_and_the_readonly_gate_are_not_props_of_this_control() {
        // Both are reserved structural attributes the RUNTIME reads off the host
        // node — the placeholder to paint the §4.16 synthetic part, `readonly` to
        // gate the mutation path. Neither changes what this component renders, so
        // decoding either here would be a value with no reader. Stated as an
        // assertion so that adding one is a deliberate act: a node carrying both
        // still decodes to the value alone.
        let props = [
            ViewProperty::new(PLACEHOLDER, ViewValue::from("type here")),
            ViewProperty::new(READONLY, ViewValue::Bool(true)),
        ];
        assert_eq!(
            decoded(&props),
            InputProps {
                value: String::new()
            }
        );
    }

    #[test]
    fn the_contract_is_the_id_the_two_tags_and_the_builder_share() {
        let contract = InputProps::contract();
        assert_eq!(contract.id, Input::COMPONENT_ID);
        assert_eq!(contract.props.len(), 1);
        assert_eq!(contract.props[0].name, InputProps::VALUE);
        assert!(contract.props[0].has_default, "value is defaulted");
        // A field's content IS its value, and the value is a prop — so there is
        // nowhere for an authored child to go. The default, restated as an
        // assertion so a change is deliberate.
        assert_eq!(
            contract.allowed_children,
            obscur_view::contract::ChildContract::None
        );
    }
}
