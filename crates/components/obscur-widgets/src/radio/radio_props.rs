#![doc = include_str!("radio_props.doc/_module.md")]

use obscur_identity::InternedSymbol;
use obscur_view::component::{
    ComponentContract, FromProps, PropsError, decode_required, prop_value,
};
use obscur_view::contract::PropContract;
use obscur_view::view::{CHECKED, StyleAttributes, ViewProperty, ViewValue};

use super::Radio;

#[doc = include_str!("radio_props.doc/RadioProps.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioProps {
    #[doc = include_str!("radio_props.doc/label.md")]
    pub label: String,
    #[doc = include_str!("radio_props.doc/selected.md")]
    pub selected: bool,
}

impl RadioProps {
    #[doc = include_str!("radio_props.doc/LABEL.md")]
    pub const LABEL: InternedSymbol = InternedSymbol::from_raw(0xF697_17FD);

    #[doc = include_str!("radio_props.doc/NAME.md")]
    const NAME: InternedSymbol = InternedSymbol::from_raw(0x5675_F954);
    #[doc = include_str!("radio_props.doc/LABEL_TYPE.md")]
    const LABEL_TYPE: InternedSymbol = InternedSymbol::from_raw(0x604F_4858);
    #[doc = include_str!("radio_props.doc/SELECTED_TYPE.md")]
    const SELECTED_TYPE: InternedSymbol = InternedSymbol::from_raw(0xC894_953D);

    #[doc = include_str!("radio_props.doc/contract.md")]
    #[must_use]
    pub fn contract() -> ComponentContract {
        ComponentContract::new(Radio::COMPONENT_ID)
            .with_name(Self::NAME)
            .with_prop(PropContract::new(Self::LABEL, Self::LABEL_TYPE).required())
            .with_prop(PropContract::new(CHECKED, Self::SELECTED_TYPE).with_default())
    }
}

impl FromProps for RadioProps {
    #[doc = include_str!("radio_props.doc/from_props.md")]
    fn from_props(props: &[ViewProperty]) -> Result<Self, PropsError> {
        let label: String = decode_required(props, Self::LABEL)?;
        let selected = match prop_value(props, Radio::PROP_SELECTED) {
            Some(&ViewValue::Bool(flag)) => flag,
            // No widget prop: this node came from markup, where the reserved
            // `checked` attribute is the only spelling — and its presence rule is
            // the cascade's, not a boolean decode's.
            _ => StyleAttributes::extract(props).is_checked(),
        };
        Ok(Self { label, selected })
    }
}

#[cfg(test)]
mod tests {
    use super::RadioProps;
    use crate::radio::Radio;
    use obscur_identity::InternedSymbol;
    use obscur_view::component::{FromProps, PropsError};
    use obscur_view::view::{CHECKED, ViewProperty, ViewValue};

    fn label(text: &str) -> ViewProperty {
        ViewProperty::new(RadioProps::LABEL, ViewValue::from(text))
    }

    #[test]
    fn a_missing_label_is_a_props_error() {
        // Required, mirroring `widget_contracts()`: a labelless radio does not
        // mount silently, it reports.
        assert_eq!(
            RadioProps::from_props(&[]),
            Err(PropsError::Missing {
                name: RadioProps::LABEL
            })
        );
    }

    #[test]
    fn a_valueless_checked_attribute_is_selected() {
        // The same trap the checkbox decoder exists for: an authored
        // `<radio checked/>` lowers to `ViewValue::Null`, which the
        // reserved-attribute rule (and `:checked`) reads as set while
        // `decode_optional::<bool>` would read as absent.
        let props = [label("A"), ViewProperty::new(CHECKED, ViewValue::Null)];
        assert_eq!(RadioProps::from_props(&props).map(|p| p.selected), Ok(true));
    }

    #[test]
    fn an_absent_checked_attribute_is_unselected() {
        assert_eq!(
            RadioProps::from_props(&[label("A")]).map(|p| p.selected),
            Ok(false)
        );
        // And an explicit `checked={false}` stays off.
        let props = [
            label("A"),
            ViewProperty::new(CHECKED, ViewValue::Bool(false)),
        ];
        assert_eq!(
            RadioProps::from_props(&props).map(|p| p.selected),
            Ok(false)
        );
    }

    #[test]
    fn the_widgets_own_prop_wins_over_the_reserved_attribute() {
        // The Rust-API path. `Radio::view` states the selection exactly, and it is
        // emitted on every node the builder makes — including the unselected ones,
        // where the reserved attribute is deliberately absent. Reading the widget
        // prop first is what keeps `Some(false)` distinguishable from "markup said
        // nothing".
        for flag in [true, false] {
            let props = [
                label("A"),
                ViewProperty::new(Radio::PROP_SELECTED, ViewValue::Bool(flag)),
            ];
            assert_eq!(
                RadioProps::from_props(&props).map(|p| p.selected),
                Ok(flag),
                "widget prop {flag}"
            );
        }
    }

    #[test]
    fn decodes_the_node_the_rust_builder_produces() {
        // The idempotence precondition: everything this decoder needs is on the
        // node `Radio::view` builds, so mounting the component on one changes
        // nothing.
        let view = Radio::new("One", InternedSymbol::from_raw(0x5678))
            .selected(true)
            .view();
        let obscur_view::view::ViewNode::Element { props, .. } = &view.root().nodes()[0] else {
            panic!("expected element root");
        };
        assert_eq!(
            RadioProps::from_props(props),
            Ok(RadioProps {
                label: "One".into(),
                selected: true,
            })
        );
    }

    #[test]
    fn the_contract_is_the_id_the_tag_and_the_builder_share() {
        let contract = RadioProps::contract();
        assert_eq!(contract.id, Radio::COMPONENT_ID);
        assert_eq!(contract.props.len(), 2);
        assert!(contract.props[0].required, "label is required");
        assert!(contract.props[1].has_default, "checked is defaulted");
        assert_eq!(
            contract.props[1].name, CHECKED,
            "the authored spelling is the reserved attribute, not the builder's \
             method name"
        );
        // A radio renders its own label, so authored children have nowhere to go —
        // the default, restated as an assertion so a change is deliberate.
        assert_eq!(
            contract.allowed_children,
            obscur_view::contract::ChildContract::None
        );
    }
}
