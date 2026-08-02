#![doc = include_str!("slot_mismatch.doc/_module.md")]

use obscur_identity::InternedSymbol;

use super::{DEFAULT_SLOT, SLOT_OUTLET, View, ViewFragment, ViewNode, slot_assignment};
use crate::component::ComponentContract;
use crate::contract::ChildContract;

#[doc = include_str!("slot_mismatch.doc/SlotMismatch.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotMismatch {
    #[doc = include_str!("slot_mismatch.doc/UndeclaredOutlet.md")]
    UndeclaredOutlet(Box<str>),
    #[doc = include_str!("slot_mismatch.doc/UndeclaredAssignment.md")]
    UndeclaredAssignment(Box<str>),
}

impl SlotMismatch {
    #[doc = include_str!("slot_mismatch.doc/find.md")]
    #[must_use]
    pub fn find(
        contract: &ComponentContract,
        output: &View,
        content: &ViewFragment,
    ) -> Vec<SlotMismatch> {
        let mut found = Vec::new();
        let mut outlets = Vec::new();
        collect_outlets(output.root().nodes(), &mut outlets);
        for name in outlets {
            if !declares(contract, &name) {
                push_once(&mut found, SlotMismatch::UndeclaredOutlet(name));
            }
        }
        // Whether a *loose* child is a mistake depends on the child contract,
        // not on this one. `ChildContract::None` is a component saying it takes
        // no content at all, and it is also the shape every Rust widget builder
        // presents: the builder installs its own output as the host node's
        // children, the reconciler retains those as slot content because the id
        // is mountable, and reporting them would report every built control
        // against itself. A child that *names* a slot is a different claim —
        // no builder writes `slot="…"`, only a person does — so that half is
        // checked whatever the child contract says.
        let loose_children_are_content = contract.allowed_children != ChildContract::None;
        for node in content {
            let named = match node {
                ViewNode::Element { props, .. } => slot_assignment(props),
                // A text run carries no properties, so it can name no slot: it
                // is content of the default slot or of nothing.
                ViewNode::Text { .. } => None,
            };
            match named {
                Some(name) => {
                    if !declares(contract, name) {
                        push_once(&mut found, SlotMismatch::UndeclaredAssignment(name.into()));
                    }
                }
                None if loose_children_are_content && !declares(contract, "") => {
                    push_once(&mut found, SlotMismatch::UndeclaredAssignment("".into()));
                }
                None => {}
            }
        }
        found
    }

    #[doc = include_str!("slot_mismatch.doc/name.md")]
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            SlotMismatch::UndeclaredOutlet(name) | SlotMismatch::UndeclaredAssignment(name) => name,
        }
    }
}

#[doc = include_str!("slot_mismatch.doc/declares.md")]
fn declares(contract: &ComponentContract, name: &str) -> bool {
    let declared = slot_symbol(name);
    contract.slots.iter().any(|slot| slot.name == declared)
}

#[doc = include_str!("slot_mismatch.doc/slot_symbol.md")]
#[must_use]
pub const fn slot_symbol(name: &str) -> InternedSymbol {
    if name.is_empty() {
        DEFAULT_SLOT
    } else {
        InternedSymbol::named(name)
    }
}

#[doc = include_str!("slot_mismatch.doc/collect_outlets.md")]
fn collect_outlets(nodes: &[ViewNode], out: &mut Vec<Box<str>>) {
    for node in nodes {
        let ViewNode::Element {
            component,
            props,
            children,
            ..
        } = node
        else {
            continue;
        };
        if *component == SLOT_OUTLET {
            let name: Box<str> = slot_assignment(props).unwrap_or("").into();
            if !out.contains(&name) {
                out.push(name);
            }
        }
        // An outlet's own children are its fallback content, and a fallback may
        // hold a further outlet; the walk does not stop at the outlet it found.
        collect_outlets(children.nodes(), out);
    }
}

#[doc = include_str!("slot_mismatch.doc/push_once.md")]
fn push_once(found: &mut Vec<SlotMismatch>, mismatch: SlotMismatch) {
    if !found.contains(&mismatch) {
        found.push(mismatch);
    }
}

#[cfg(test)]
mod tests {
    use super::{SlotMismatch, slot_symbol};
    use crate::component::ComponentContract;
    use crate::contract::{ChildContract, SlotCardinality, SlotContract};
    use crate::view::{DEFAULT_SLOT, SLOT, View, ViewFragment, ViewNode, ViewProperty, ViewValue};
    use obscur_identity::ComponentId;

    const BOX: ComponentId = ComponentId::from_raw(0x00C0_0001);

    /// A contract taking children and declaring exactly `slots`.
    fn contract(slots: &[&str]) -> ComponentContract {
        let mut contract =
            ComponentContract::new(ComponentId::from_raw(1)).allowing_children(ChildContract::Any);
        for name in slots {
            contract = contract.with_slot(SlotContract::new(
                slot_symbol(name),
                SlotCardinality::Many,
                ChildContract::Any,
            ));
        }
        contract
    }

    /// A render output holding one outlet per name in `outlets`.
    fn output(outlets: &[&str]) -> View {
        View::from_fragment(ViewFragment::from_nodes(
            outlets
                .iter()
                .map(|name| {
                    ViewNode::element(BOX)
                        .with_children(ViewFragment::from_nodes(vec![ViewNode::slot(*name)]))
                })
                .collect(),
        ))
    }

    fn assigned(slot: &str) -> ViewNode {
        ViewNode::element(BOX).with_prop(ViewProperty::new(SLOT, ViewValue::Text(slot.into())))
    }

    #[test]
    fn a_contract_that_declares_every_outlet_it_renders_reports_nothing() {
        let found = SlotMismatch::find(
            &contract(&["header", "body"]),
            &output(&["header", "body"]),
            &ViewFragment::from_nodes(vec![assigned("header")]),
        );
        assert_eq!(found, Vec::new());
    }

    #[test]
    fn a_misspelled_outlet_is_caught_against_the_contract() {
        // The whole point: `<slot name="hedaer"/>` used to receive nothing, in
        // silence, for as long as the typo lived.
        let found = SlotMismatch::find(
            &contract(&["header"]),
            &output(&["hedaer"]),
            &ViewFragment::new(),
        );
        assert_eq!(found, vec![SlotMismatch::UndeclaredOutlet("hedaer".into())]);
        assert_eq!(found[0].name(), "hedaer");
    }

    #[test]
    fn a_misspelled_assignment_is_caught_from_the_authors_side() {
        // The same fault, written by the other person: the author's node names a
        // slot the component never declared, so it is dropped on projection.
        let found = SlotMismatch::find(
            &contract(&["header"]),
            &output(&["header"]),
            &ViewFragment::from_nodes(vec![assigned("hedaer")]),
        );
        assert_eq!(
            found,
            vec![SlotMismatch::UndeclaredAssignment("hedaer".into())]
        );
    }

    #[test]
    fn the_default_slot_is_declared_under_its_reserved_name() {
        let declaring = contract(&[""]);
        assert_eq!(declaring.slots[0].name, DEFAULT_SLOT);
        assert_eq!(
            SlotMismatch::find(&declaring, &output(&[""]), &ViewFragment::new()),
            Vec::new()
        );
        // And a component that does NOT declare it, rendering the unnamed
        // outlet anyway, is reported under the empty name.
        assert_eq!(
            SlotMismatch::find(&contract(&["header"]), &output(&[""]), &ViewFragment::new()),
            vec![SlotMismatch::UndeclaredOutlet("".into())]
        );
    }

    #[test]
    fn unassigned_content_is_measured_against_the_default_slot() {
        // A text run can name no slot, so it is default content or nothing —
        // and a component with no default slot silently drops it.
        let loose = ViewFragment::from_nodes(vec![ViewNode::text("Save")]);
        assert_eq!(
            SlotMismatch::find(&contract(&["header"]), &output(&["header"]), &loose),
            vec![SlotMismatch::UndeclaredAssignment("".into())]
        );
        assert_eq!(
            SlotMismatch::find(&contract(&["", "header"]), &output(&["header"]), &loose),
            Vec::new()
        );
    }

    #[test]
    fn a_component_taking_no_children_is_not_asked_about_the_loose_ones_it_drops() {
        // Every Rust widget builder installs its own output as the host node's
        // children, and the reconciler retains those as slot content because the
        // id is mountable. Reporting them would report every built control
        // against itself; `ChildContract::None` is the component saying so.
        let refusing = ComponentContract::new(ComponentId::from_raw(1));
        assert_eq!(refusing.allowed_children, ChildContract::None);
        let found = SlotMismatch::find(
            &refusing,
            &View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::text("✓")])),
            &ViewFragment::from_nodes(vec![ViewNode::text("Agree"), ViewNode::element(BOX)]),
        );
        assert_eq!(found, Vec::new());
    }

    #[test]
    fn a_named_assignment_is_checked_however_the_child_contract_reads() {
        // The discriminator: no builder ever writes `slot="…"`, only a person
        // does. So a named assignment is always a claim about a slot, and a
        // claim about a slot the contract does not declare is always a drop
        // worth reporting — even on a component that takes no children.
        let refusing = ComponentContract::new(ComponentId::from_raw(1));
        assert_eq!(refusing.allowed_children, ChildContract::None);
        assert_eq!(
            SlotMismatch::find(
                &refusing,
                &View::from_fragment(ViewFragment::new()),
                &ViewFragment::from_nodes(vec![assigned("header")]),
            ),
            vec![SlotMismatch::UndeclaredAssignment("header".into())]
        );
    }

    #[test]
    fn each_distinct_name_is_reported_once_however_often_it_appears() {
        let found = SlotMismatch::find(
            &contract(&["header"]),
            &output(&["hedaer", "hedaer"]),
            &ViewFragment::from_nodes(vec![assigned("nope"), assigned("nope")]),
        );
        assert_eq!(
            found,
            vec![
                SlotMismatch::UndeclaredOutlet("hedaer".into()),
                SlotMismatch::UndeclaredAssignment("nope".into()),
            ]
        );
    }

    #[test]
    fn an_outlets_fallback_is_walked_for_further_outlets() {
        let nested = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::slot("header")
                .with_children(ViewFragment::from_nodes(vec![ViewNode::slot("hedaer")])),
        ]));
        assert_eq!(
            SlotMismatch::find(&contract(&["header"]), &nested, &ViewFragment::new()),
            vec![SlotMismatch::UndeclaredOutlet("hedaer".into())]
        );
    }

    #[test]
    fn a_component_with_no_outlets_and_no_content_is_free() {
        let plain = View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::text("card:Hi")]));
        assert_eq!(
            SlotMismatch::find(&contract(&[]), &plain, &ViewFragment::new()),
            Vec::new()
        );
    }

    #[test]
    fn a_slot_symbol_never_aliases_the_default_one() {
        // The default slot is a reserved low symbol and a named one always
        // carries the high bit, so no author-written name can ever collide with
        // the slot that has no name.
        assert_eq!(slot_symbol(""), DEFAULT_SLOT);
        assert_eq!(slot_symbol("").raw() & 0x8000_0000, 0);
        for name in ["header", "body", "default", "actions"] {
            assert_ne!(slot_symbol(name), DEFAULT_SLOT);
            assert_ne!(slot_symbol(name).raw() & 0x8000_0000, 0, "{name}");
        }
    }
}
