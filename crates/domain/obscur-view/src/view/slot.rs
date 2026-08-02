#![doc = include_str!("slot.doc/_module.md")]

use obscur_identity::{ComponentId, InternedSymbol};

use super::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("slot.doc/SLOT.md")]
pub const SLOT: InternedSymbol = InternedSymbol::from_raw(0x0000_0301);

#[doc = include_str!("slot.doc/SLOT_OUTLET.md")]
pub const SLOT_OUTLET: ComponentId = ComponentId::from_raw(0x00B0_0001);

#[doc = include_str!("slot.doc/DEFAULT_SLOT.md")]
pub const DEFAULT_SLOT: InternedSymbol = InternedSymbol::from_raw(0x0000_0302);

#[doc = include_str!("slot.doc/FIRST_MOUNTABLE_COMPONENT.md")]
const FIRST_MOUNTABLE_COMPONENT: u64 = 0x00D0_0000;

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

#[doc = include_str!("slot.doc/may_project_content.md")]
#[must_use]
pub fn may_project_content(component: ComponentId) -> bool {
    component.raw() >= FIRST_MOUNTABLE_COMPONENT
}

#[doc = include_str!("slot.doc/outlet_name.md")]
fn outlet_name(node: &ViewNode) -> &str {
    match node {
        ViewNode::Element {
            component, props, ..
        } if *component == SLOT_OUTLET => slot_assignment(props).unwrap_or(""),
        _ => "",
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
            let name = outlet_name(&node);
            let before = projected.len();
            projected.extend(
                content
                    .iter()
                    .filter(|candidate| assigned_to(candidate, name))
                    .cloned(),
            );
            // Fallback content: an outlet nothing filled renders its own
            // children instead. That is what lets one outlet state both halves
            // of "the author's content, or this if there is none" —
            // `<button label="Save"/>` and `<button>Save</button>` are the same
            // component rendering the same single position.
            if projected.len() == before
                && let ViewNode::Element { children, .. } = node
            {
                projected.extend(project_fragment(children, content).into_nodes());
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
        ViewNode::Element { props, .. } => match slot_assignment(props) {
            Some(assigned) => assigned == name,
            // Content that names no slot belongs to the default one, and to no
            // named one — the rule that lets `<button>Save</button>` keep its
            // author's text without the author naming a slot.
            None => name.is_empty(),
        },
        // A text run carries no properties, so it can name no slot: it is
        // content of the default slot or of nothing.
        ViewNode::Text { .. } => name.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SLOT, SLOT_OUTLET, assigns_slots, may_project_content, project_slots, slot_assignment,
    };
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
        // A child naming no slot is content of the DEFAULT slot, and of no named
        // one: neither region here declares the default outlet, so the loose
        // child reaches neither. Naming a slot is how an author opts out of the
        // default; declaring the unnamed outlet is how a component opts in.
        let content = ViewFragment::from_nodes(vec![
            ViewNode::element(BOX)
                .with_children(ViewFragment::from_nodes(vec![ViewNode::text("loose")])),
        ]);
        let projected = project_slots(two_regions(), &content);
        assert!(marks_under(&projected, 0).is_empty());
        assert!(marks_under(&projected, 1).is_empty());
    }

    #[test]
    fn the_default_outlet_takes_the_content_that_names_no_slot() {
        // The whole reason the default slot exists: a text run carries no
        // properties, so `<button>Save</button>`'s only child can never name a
        // slot — and a control that could not project it would eat the author's
        // markup.
        let output = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(BOX)
                .with_children(ViewFragment::from_nodes(vec![ViewNode::slot("header")])),
            ViewNode::element(BOX)
                .with_children(ViewFragment::from_nodes(vec![ViewNode::slot("")])),
        ]));
        let content = ViewFragment::from_nodes(vec![
            assigned("header", "H"),
            ViewNode::text("Save"),
            ViewNode::element(BOX)
                .with_children(ViewFragment::from_nodes(vec![ViewNode::text("loose")])),
        ]);
        let projected = project_slots(output, &content);

        assert_eq!(marks_under(&projected, 0), ["H"]);
        assert_eq!(
            marks_under(&projected, 1),
            ["Save", "loose"],
            "the text run and the unassigned element, in authored order — and \
             not the child that asked for `header`"
        );
    }

    #[test]
    fn an_outlet_nothing_fills_renders_its_fallback() {
        // The other half of one position: `<button label="Save"/>` names no
        // content, so the outlet's own children are what renders. An outlet the
        // author DID fill drops its fallback rather than showing both.
        let output = || {
            View::from_fragment(ViewFragment::from_nodes(vec![
                ViewNode::element(BOX).with_children(ViewFragment::from_nodes(vec![
                    ViewNode::slot("")
                        .with_children(ViewFragment::from_nodes(vec![ViewNode::text("fallback")])),
                ])),
            ]))
        };
        assert_eq!(
            marks_under(&project_slots(output(), &ViewFragment::new()), 0),
            ["fallback"]
        );
        let content = ViewFragment::from_nodes(vec![ViewNode::text("Save")]);
        assert_eq!(marks_under(&project_slots(output(), &content), 0), ["Save"]);
    }

    #[test]
    fn only_a_mountable_id_retains_its_content() {
        // The reconciler's second gate. A primitive and the outlet itself can
        // host no component, so their children are never anyone's slot content;
        // a widget and a user component can, and pay one clone for it.
        assert!(!may_project_content(BOX));
        assert!(!may_project_content(SLOT_OUTLET));
        assert!(may_project_content(ComponentId::from_raw(0x00D0_0001)));
        assert!(may_project_content(ComponentId::from_raw(
            0x8000_0000_0000_0001
        )));
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
