#![doc = include_str!("button_props.doc/_module.md")]

use obscur_identity::InternedSymbol;
use obscur_view::component::{ComponentContract, FromProps, PropsError, decode_optional};
use obscur_view::contract::{ChildContract, PropContract, SlotCardinality, SlotContract};
use obscur_view::view::{DEFAULT_SLOT, ViewProperty};

use super::Button;

#[doc = include_str!("button_props.doc/ButtonProps.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonProps {
    #[doc = include_str!("button_props.doc/label.md")]
    pub label: String,
}

impl ButtonProps {
    #[doc = include_str!("button_props.doc/LABEL.md")]
    pub const LABEL: InternedSymbol = InternedSymbol::from_raw(0xF697_17FD);

    #[doc = include_str!("button_props.doc/NAME.md")]
    const NAME: InternedSymbol = InternedSymbol::from_raw(0x3388_1A91);
    #[doc = include_str!("button_props.doc/LABEL_TYPE.md")]
    const LABEL_TYPE: InternedSymbol = InternedSymbol::from_raw(0x604F_4858);

    #[doc = include_str!("button_props.doc/contract.md")]
    #[must_use]
    pub fn contract() -> ComponentContract {
        ComponentContract::new(Button::COMPONENT_ID)
            .with_name(Self::NAME)
            .with_prop(PropContract::new(Self::LABEL, Self::LABEL_TYPE).with_default())
            .with_slot(SlotContract::new(
                DEFAULT_SLOT,
                SlotCardinality::Many,
                ChildContract::Any,
            ))
            .allowing_children(ChildContract::Any)
    }
}

impl FromProps for ButtonProps {
    #[doc = include_str!("button_props.doc/from_props.md")]
    fn from_props(props: &[ViewProperty]) -> Result<Self, PropsError> {
        let label: Option<String> = decode_optional(props, Self::LABEL)?;
        Ok(Self {
            label: label.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ButtonProps;
    use crate::button::Button;
    use obscur_identity::InternedSymbol;
    use obscur_view::component::FromProps;
    use obscur_view::contract::{ChildContract, SlotCardinality};
    use obscur_view::view::{DEFAULT_SLOT, ViewNode, ViewProperty, ViewValue};

    #[test]
    fn a_labelless_button_decodes_to_an_empty_label() {
        // `<button>Save</button>` states its text as a CHILD, so a button with no
        // `label` prop is the ordinary case, not a broken one — nothing here may
        // report. (A checkbox reports a missing label; it renders its own, and
        // has nowhere for an authored child to go.)
        assert_eq!(
            ButtonProps::from_props(&[]),
            Ok(ButtonProps {
                label: String::new()
            })
        );
    }

    #[test]
    fn an_authored_label_attribute_is_the_buttons_text() {
        let props = [ViewProperty::new(
            ButtonProps::LABEL,
            ViewValue::from("Save"),
        )];
        assert_eq!(
            ButtonProps::from_props(&props).map(|p| p.label),
            Ok("Save".to_owned())
        );
    }

    #[test]
    fn decodes_the_node_the_rust_builder_produces() {
        // The idempotence precondition, and for a button it reads the other way
        // round from every other control's: `Button::view` states the label as a
        // text CHILD and emits no `label` prop at all, so this decodes to the
        // empty label — and the child is what the mounted component projects.
        // Anything else would render the label twice.
        let view = Button::new("Save", InternedSymbol::from_raw(0xABCD)).view();
        let ViewNode::Element {
            props, children, ..
        } = &view.root().nodes()[0]
        else {
            panic!("expected element root");
        };
        assert_eq!(
            ButtonProps::from_props(props),
            Ok(ButtonProps {
                label: String::new()
            })
        );
        assert_eq!(children.len(), 1, "the label is the child");
    }

    #[test]
    fn the_contract_is_the_id_the_tag_and_the_builder_share() {
        let contract = ButtonProps::contract();
        assert_eq!(contract.id, Button::COMPONENT_ID);
        assert_eq!(contract.props.len(), 1);
        assert_eq!(contract.props[0].name, ButtonProps::LABEL);
        assert!(
            contract.props[0].has_default,
            "an authored `<button>Save</button>` names no label prop"
        );
    }

    #[test]
    fn the_contract_declares_the_default_slot_and_takes_children() {
        // The first shipped component to declare a slot at all. The two halves
        // must agree: a contract that declared no slot while the component
        // rendered an outlet would be a §2.13 export that lies about what the
        // control does with an author's markup.
        let contract = ButtonProps::contract();
        assert_eq!(contract.slots.len(), 1);
        assert_eq!(contract.slots[0].name, DEFAULT_SLOT);
        assert_eq!(contract.slots[0].cardinality, SlotCardinality::Many);
        assert_eq!(contract.slots[0].allowed_children, ChildContract::Any);
        assert_eq!(
            contract.allowed_children,
            ChildContract::Any,
            "a button's children are its content — the one shipped control of \
             which that is true"
        );
    }
}
