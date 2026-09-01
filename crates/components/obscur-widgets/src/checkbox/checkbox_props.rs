#![doc = include_str!("checkbox_props.doc/_module.md")]

use obscur_identity::InternedSymbol;
use obscur_view::component::{
    ComponentContract, FromProps, PropsError, decode_required, prop_value,
};
use obscur_view::contract::PropContract;
use obscur_view::view::{CHECKED, StyleAttributes, ViewProperty, ViewValue};

use super::Checkbox;

#[doc = include_str!("checkbox_props.doc/CheckboxProps.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckboxProps {
    #[doc = include_str!("checkbox_props.doc/label.md")]
    pub label: String,
    #[doc = include_str!("checkbox_props.doc/checked.md")]
    pub checked: Option<bool>,
}

impl CheckboxProps {
    #[doc = include_str!("checkbox_props.doc/LABEL.md")]
    pub const LABEL: InternedSymbol = InternedSymbol::from_raw(0xF697_17FD);

    #[doc = include_str!("checkbox_props.doc/NAME.md")]
    const NAME: InternedSymbol = InternedSymbol::from_raw(0x000B_8638);
    #[doc = include_str!("checkbox_props.doc/LABEL_TYPE.md")]
    const LABEL_TYPE: InternedSymbol = InternedSymbol::from_raw(0x604F_4858);
    #[doc = include_str!("checkbox_props.doc/CHECKED_TYPE.md")]
    const CHECKED_TYPE: InternedSymbol = InternedSymbol::from_raw(0xD0C8_231A);

    #[doc = include_str!("checkbox_props.doc/contract.md")]
    #[must_use]
    pub fn contract() -> ComponentContract {
        ComponentContract::new(Checkbox::COMPONENT_ID)
            .with_name(Self::NAME)
            .with_prop(PropContract::new(Self::LABEL, Self::LABEL_TYPE).required())
            .with_prop(PropContract::new(CHECKED, Self::CHECKED_TYPE).with_default())
    }
}

impl FromProps for CheckboxProps {
    #[doc = include_str!("checkbox_props.doc/from_props.md")]
    fn from_props(props: &[ViewProperty]) -> Result<Self, PropsError> {
        let label: String = decode_required(props, Self::LABEL, "label")?;
        let checked = match prop_value(props, Checkbox::PROP_CHECKED) {
            Some(&ViewValue::Int(-1)) => None,
            Some(&ViewValue::Int(state)) => Some(state != 0),
            // No tri-state prop: this node came from markup, where the reserved
            // `checked` attribute is the only spelling — and its presence rule is
            // the cascade's, not a boolean decode's.
            _ => Some(StyleAttributes::extract(props).is_checked()),
        };
        Ok(Self { label, checked })
    }
}

#[cfg(test)]
mod tests {
    use super::CheckboxProps;
    use crate::checkbox::Checkbox;
    use obscur_identity::InternedSymbol;
    use obscur_view::component::{FromProps, PropsError};
    use obscur_view::view::{CHECKED, ViewProperty, ViewValue};

    fn label(text: &str) -> ViewProperty {
        ViewProperty::new(CheckboxProps::LABEL, ViewValue::from(text))
    }

    #[test]
    fn a_missing_label_is_a_props_error() {
        // Required, mirroring `widget_contracts()`: a labelless checkbox does not
        // mount silently, it reports.
        assert_eq!(
            CheckboxProps::from_props(&[]),
            Err(PropsError::missing(CheckboxProps::LABEL, "label"))
        );
    }

    #[test]
    fn a_valueless_checked_attribute_is_checked() {
        // THE trap this decoder exists for. An authored `<checkbox checked/>`
        // lowers to `ViewValue::Null`; the reserved-attribute rule reads presence
        // as set (and so does `:checked`), while `decode_optional::<bool>` — what
        // `#[derive(Contract)]` would emit — reads it as absent. The two must not
        // disagree on one node.
        let props = [label("A"), ViewProperty::new(CHECKED, ViewValue::Null)];
        assert_eq!(
            CheckboxProps::from_props(&props).map(|p| p.checked),
            Ok(Some(true))
        );
    }

    #[test]
    fn an_absent_checked_attribute_is_unchecked() {
        assert_eq!(
            CheckboxProps::from_props(&[label("A")]).map(|p| p.checked),
            Ok(Some(false))
        );
        // And an explicit `checked={false}` stays off.
        let props = [
            label("A"),
            ViewProperty::new(CHECKED, ViewValue::Bool(false)),
        ];
        assert_eq!(
            CheckboxProps::from_props(&props).map(|p| p.checked),
            Ok(Some(false))
        );
    }

    #[test]
    fn the_widgets_own_tri_state_prop_wins_and_carries_mixed() {
        // The Rust-API path: `Checkbox::view` states the tri-state exactly, and it
        // is the only spelling that can say *mixed* — markup cannot.
        for (encoded, expected) in [(-1, None), (0, Some(false)), (1, Some(true))] {
            let props = [
                label("A"),
                ViewProperty::new(Checkbox::PROP_CHECKED, ViewValue::Int(encoded)),
            ];
            assert_eq!(
                CheckboxProps::from_props(&props).map(|p| p.checked),
                Ok(expected),
                "tri-state {encoded}"
            );
        }
    }

    #[test]
    fn decodes_the_node_the_rust_builder_produces() {
        // The idempotence precondition: everything this decoder needs is on the
        // node `Checkbox::view` builds, so mounting the component on one changes
        // nothing.
        let view = Checkbox::new("Agree", InternedSymbol::from_raw(0x1234))
            .checked(true)
            .view();
        let obscur_view::view::ViewNode::Element { props, .. } = &view.root().nodes()[0] else {
            panic!("expected element root");
        };
        assert_eq!(
            CheckboxProps::from_props(props),
            Ok(CheckboxProps {
                label: "Agree".into(),
                checked: Some(true),
            })
        );
    }

    #[test]
    fn the_contract_is_the_id_the_tag_and_the_builder_share() {
        let contract = CheckboxProps::contract();
        assert_eq!(contract.id, Checkbox::COMPONENT_ID);
        assert_eq!(contract.props.len(), 2);
        assert!(contract.props[0].required, "label is required");
        assert!(contract.props[1].has_default, "checked is defaulted");
        assert_eq!(contract.props[1].name, CHECKED);
        // A checkbox renders its own label, so authored children have nowhere to
        // go — the default, restated as an assertion so a change is deliberate.
        assert_eq!(
            contract.allowed_children,
            obscur_view::contract::ChildContract::None
        );
    }
}
