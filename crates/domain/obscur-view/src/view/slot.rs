#![doc = include_str!("slot.doc/_module.md")]

use obscur_identity::{ComponentId, InternedSymbol};

use super::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("slot.doc/SLOT.md")]
pub const SLOT: InternedSymbol = InternedSymbol::from_raw(0x0000_0301);

#[doc = include_str!("slot.doc/SLOT_OUTLET.md")]
pub const SLOT_OUTLET: ComponentId = ComponentId::from_raw(0x00B0_0001);

#[doc = include_str!("slot.doc/slot_assignment.md")]
#[must_use]
pub fn slot_assignment(props: &[ViewProperty]) -> Option<&str> {
    props
        .iter()
        .find(|prop| prop.name() == SLOT)
        .and_then(|prop| match prop.value() {
            ViewValue::Text(name) => Some(&**name),
            _ => None,
        })
}

#[doc = include_str!("slot.doc/assigns_slots.md")]
#[must_use]
pub fn assigns_slots(nodes: &[ViewNode]) -> bool {
    nodes.iter().any(|node| match node {
        ViewNode::Element { props, .. } => slot_assignment(props).is_some(),
        ViewNode::Text { .. } => false,
    })
}

#[doc = include_str!("slot.doc/outlet_name.md")]
fn outlet_name(node: &ViewNode) -> Option<&str> {
    match node {
        ViewNode::Element {
            component, props, ..
        } if *component == SLOT_OUTLET => slot_assignment(props),
        _ => None,
    }
}

#[doc = include_str!("slot.doc/is_outlet.md")]
#[must_use]
pub fn is_outlet(node: &ViewNode) -> bool {
    matches!(node, ViewNode::Element { component, .. } if *component == SLOT_OUTLET)
}

#[doc = include_str!("slot.doc/has_outlet.md")]
#[must_use]
pub fn has_outlet(nodes: &[ViewNode]) -> bool {
    nodes.iter().any(|node| match node {
        ViewNode::Element { children, .. } => is_outlet(node) || has_outlet(children.nodes()),
        ViewNode::Text { .. } => false,
    })
}

#[doc = include_str!("slot.doc/project_slots.md")]
#[must_use]
pub fn project_slots(output: View, content: &ViewFragment) -> View {
    if !has_outlet(output.root().nodes()) {
        return output;
    }
    View::from_fragment(project_fragment(output.into_fragment(), content))
}

#[doc = include_str!("slot.doc/project_fragment.md")]
fn project_fragment(fragment: ViewFragment, content: &ViewFragment) -> ViewFragment {
    let mut projected: Vec<ViewNode> = Vec::with_capacity(fragment.len());
    for node in fragment.into_nodes() {
        if is_outlet(&node) {
            // An unnamed outlet names no slot, so nothing is assigned to it.
            if let Some(name) = outlet_name(&node) {
                projected.extend(
                    content
                        .iter()
                        .filter(|candidate| assigned_to(candidate, name))
                        .cloned(),
                );
            }
        } else {
            projected.push(project_node(node, content));
        }
    }
    ViewFragment::from_nodes(projected)
}

#[doc = include_str!("slot.doc/project_node.md")]
fn project_node(node: ViewNode, content: &ViewFragment) -> ViewNode {
    match node {
        ViewNode::Element {
            component,
            key,
            props,
            events,
            children,
        } => ViewNode::Element {
            component,
            key,
            props,
            events,
            children: project_fragment(children, content),
        },
        text @ ViewNode::Text { .. } => text,
    }
}

#[doc = include_str!("slot.doc/assigned_to.md")]
fn assigned_to(node: &ViewNode, name: &str) -> bool {
    match node {
        ViewNode::Element { props, .. } => slot_assignment(props) == Some(name),
        ViewNode::Text { .. } => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{SLOT, SLOT_OUTLET, assigns_slots, project_slots, slot_assignment};
    use crate::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};
    use obscur_identity::ComponentId;

    const BOX: ComponentId = ComponentId::from_raw(0x00C0_0001);

    /// An element assigned to `slot`, carrying `mark` as its text child so the
    /// assertions can tell two of them apart.
    fn assigned(slot: &str, mark: &str) -> ViewNode {
        ViewNode::element(BOX)
            .with_prop(ViewProperty::new(SLOT, ViewValue::Text(slot.into())))
            .with_children(ViewFragment::from_nodes(vec![ViewNode::text(mark)]))
    }

    /// The text marks beneath the `index`-th top-level node of `view`.
    fn marks_under(view: &View, index: usize) -> Vec<String> {
        let ViewNode::Element { children, .. } = &view.root().nodes()[index] else {
            panic!("a region is an element");
        };
        children
            .nodes()
            .iter()
            .map(|node| match node {
                ViewNode::Element { children, .. } => match &children.nodes()[0] {
                    ViewNode::Text { content, .. } => content.to_string(),
                    ViewNode::Element { .. } => panic!("a mark is text"),
                },
                ViewNode::Text { content, .. } => content.to_string(),
            })
            .collect()
    }

    /// Two named regions, each holding one outlet — the shape a component that
    /// takes children actually renders.
    fn two_regions() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(BOX)
                .with_children(ViewFragment::from_nodes(vec![ViewNode::slot("header")])),
            ViewNode::element(BOX)
                .with_children(ViewFragment::from_nodes(vec![ViewNode::slot("body")])),
        ]))
    }

    #[test]
    fn content_reaches_the_outlet_that_names_its_slot_and_no_other() {
        let content =
            ViewFragment::from_nodes(vec![assigned("header", "H"), assigned("body", "B")]);
        let projected = project_slots(two_regions(), &content);

        assert_eq!(marks_under(&projected, 0), ["H"]);
        assert_eq!(marks_under(&projected, 1), ["B"]);
    }

    #[test]
    fn a_many_slot_takes_every_child_that_names_it_in_order() {
        let content = ViewFragment::from_nodes(vec![
            assigned("body", "one"),
            assigned("header", "H"),
            assigned("body", "two"),
        ]);
        let projected = project_slots(two_regions(), &content);

        assert_eq!(marks_under(&projected, 0), ["H"]);
        assert_eq!(
            marks_under(&projected, 1),
            ["one", "two"],
            "authored order is the order within a slot"
        );
    }

    #[test]
    fn an_outlet_nothing_fills_leaves_no_node_behind() {
        // The reason an outlet is substituted rather than styled away: an unfilled
        // slot must not become an empty box that lays out and paints.
        let projected = project_slots(two_regions(), &ViewFragment::new());
        assert!(marks_under(&projected, 0).is_empty());
        assert!(marks_under(&projected, 1).is_empty());
        let ViewNode::Element { children, .. } = &projected.root().nodes()[0] else {
            panic!("a region is an element");
        };
        assert!(
            children.is_empty(),
            "the outlet itself is gone, not emptied"
        );
    }

    #[test]
    fn an_unassigned_child_reaches_no_outlet() {
        // §2.12 declares *named* slots. A child naming none is not "the default
        // one" — there is no default one.
        let content = ViewFragment::from_nodes(vec![
            ViewNode::element(BOX)
                .with_children(ViewFragment::from_nodes(vec![ViewNode::text("loose")])),
        ]);
        let projected = project_slots(two_regions(), &content);
        assert!(marks_under(&projected, 0).is_empty());
        assert!(marks_under(&projected, 1).is_empty());
    }

    #[test]
    fn an_output_with_no_outlet_is_returned_unchanged() {
        // The early-out that keeps every childless component paying nothing.
        let output = View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::text("card:Hi")]));
        let content = ViewFragment::from_nodes(vec![assigned("header", "H")]);
        assert_eq!(project_slots(output.clone(), &content), output);
    }

    #[test]
    fn assigns_slots_gates_on_a_real_assignment() {
        assert!(assigns_slots(&[assigned("header", "H")]));
        assert!(!assigns_slots(&[
            ViewNode::element(BOX),
            ViewNode::text("x")
        ]));
        assert!(!assigns_slots(&[]));
    }

    #[test]
    fn the_slot_constructor_names_the_outlet_id_and_the_reserved_symbol() {
        let ViewNode::Element {
            component, props, ..
        } = ViewNode::slot("header")
        else {
            panic!("an outlet is an element");
        };
        assert_eq!(component, SLOT_OUTLET);
        assert_eq!(slot_assignment(&props), Some("header"));
    }

    #[test]
    fn the_outlet_id_aliases_no_other_namespace() {
        // Disjoint by construction: every primitive is `0x00C0_xxxx`, every widget
        // `0x00D0_xxxx`, and every user component carries the top bit.
        assert_eq!(SLOT_OUTLET.raw() & 0x8000_0000_0000_0000, 0);
        assert_ne!(SLOT_OUTLET, BOX);
        assert!(SLOT_OUTLET.raw() < 0x00C0_0000);
    }
}
