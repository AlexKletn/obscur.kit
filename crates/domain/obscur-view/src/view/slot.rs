//! Slots: the reserved `slot="…"` assignment, the outlet a component's output
//! declares, and the projection that joins them (spec §2.12, §2.13).
//!
//! # What §2.12 asks for
//!
//! > Components may declare: named slots; allowed child contracts; slot
//! > cardinality; exported style parts; exported visual states.
//!
//! and §2.13 lists `slots` among the fields of the contract a component
//! **exports**. So a slot is a *declaration*, and a declaration is only
//! load-bearing if something acts on it. Two halves have to meet:
//!
//! * the **assignment** — an authored `<h1 slot="header"/>` under a component
//!   tag. Both front ends already lower this, to an ordinary property under the
//!   reserved [`SLOT`] symbol, and both already check it against the
//!   component's declared slots at compile time (name, cardinality, and the
//!   slot's own child rule);
//! * the **outlet** — the position *inside the component's own output* where
//!   that content belongs. This is the half that did not exist: a component's
//!   output named no such position, so there was nowhere to route to and the
//!   authored children were dropped.
//!
//! [`ViewNode::slot`](super::ViewNode::slot) is that position, and
//! [`project_slots`] is the join.
//!
//! # Named slots only
//!
//! §2.12 says *named* slots, and says nothing about an unnamed default one.
//! An outlet therefore always names a slot, and content is matched to it by
//! name and by nothing else. A child carrying no `slot="…"` reaches no outlet,
//! and an outlet no child names projects to nothing rather than to a leftover
//! box. Declaration order is not a destination — inventing one is what
//! `5e8e7fe` refused for `<Loading>`.
//!
//! # The authored spelling
//!
//! `<slot name="header"/>` is the outlet, on both front ends. It needs no node of
//! its own in either parser: it is an ordinary element tag carrying an ordinary
//! attribute, and the whole of it is two rules in the frontend crate's
//! `name_binding` — the tag binds to [`SLOT_OUTLET`] and, *on that tag only*,
//! `name` binds to [`SLOT`]. A component's render is a `View`, and `okui!`
//! produces a `View`, so `okui! { <slot name="header"/> }` inside
//! `Component::render` is the same outlet
//! [`ViewNode::slot`](super::ViewNode::slot) builds by hand.
//!
//! # Why an outlet is a reserved `ComponentId` and not a `ViewNode` variant
//!
//! Because an outlet never becomes a retained node. Projection runs over a
//! component's output *before* it is reconciled, so the outlet is substituted away
//! exactly like an `<If>` is at lowering — "these constructs do not create real
//! retained nodes unless their rendered output requires them" (§2.11). A third
//! [`ViewNode`] variant would put a case in all sixty-odd exhaustive matches over
//! the enum to describe a node no consumer of the retained tree can ever see.
//!
//! **A stray outlet is refused, not retained.** An outlet authored *outside* a
//! component — one in a view that is nobody's render output — meets no projection
//! pass, and used to land in the tree as an ordinary `Element` carrying the
//! reserved id, rendering as an empty box. The reconciler now refuses to
//! materialize any node [`is_outlet`] answers `true` for, so the sentence above is
//! unconditional on every path that builds a tree; `UiRuntime`, which has a §1.6
//! diagnostics channel, also *reports* the stray rather than dropping it quietly.
//! This paragraph said "never" once before, as a claim rather than a mechanism,
//! and a verifier built a stray and found it in the tree. What changed is the
//! mechanism, not the prose.

use obscur_identity::{ComponentId, InternedSymbol};

use super::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// Reserved symbol for a child's `slot="…"` assignment — the name of the slot
/// the child is projected into (spec §2.12).
///
/// Deliberately **not** a [`StyleAttributes`](super::StyleAttributes) member: a
/// slot assignment is structure, not a CSS selector input, so `extract` never
/// sees it and no selector can match on it. It lives in the node's ordinary
/// property list, which is where the reconciler and [`project_slots`] read it —
/// the same shape [`TEST_ID`](super::TEST_ID) uses, and for the same reason.
///
/// It occupies a namespace of its own (`0x0000_03xx`), distinct from the
/// CSS-relevant reserved attributes (`0x0000_000x`), the reserved events
/// (`0x0000_010x`) and the stable selectors (`0x0000_02xx`).
pub const SLOT: InternedSymbol = InternedSymbol::from_raw(0x0000_0301);

/// Reserved [`ComponentId`] of a **slot outlet** — the marker a component's
/// output uses to say "the content assigned to this slot goes here".
///
/// Nothing is ever registered under this id and no node ever retains it:
/// [`project_slots`] replaces every outlet with the content assigned to its
/// name before the output reaches the reconciler.
///
/// It sits in a reserved range of its own (`0x00B0_xxxx`), below the primitive
/// (`0x00C0_xxxx`) and widget (`0x00D0_xxxx`) ranges and clear of the top-bit
/// namespace `user_component_id` hashes into, so it can alias neither a shipped
/// component nor an author's.
pub const SLOT_OUTLET: ComponentId = ComponentId::from_raw(0x00B0_0001);

/// The slot a node's props assign it to — the value of its `slot="…"` — or
/// [`None`] when it carries no assignment (or one that is not text).
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

/// Whether any node in `nodes` carries a `slot="…"` assignment.
///
/// The reconciler's gate for retaining a component host's authored children:
/// the desired tree is gone by the time a component renders, so the content has
/// to be kept somewhere, and keeping *every* element's children would duplicate
/// the whole desired tree once per level of depth. A document that authors no
/// slot therefore pays nothing, and one that does pays for exactly the child
/// lists it assigned.
#[must_use]
pub fn assigns_slots(nodes: &[ViewNode]) -> bool {
    nodes.iter().any(|node| match node {
        ViewNode::Element { props, .. } => slot_assignment(props).is_some(),
        ViewNode::Text { .. } => false,
    })
}

/// The slot name this node is an outlet for, or [`None`] when it is not an
/// outlet.
fn outlet_name(node: &ViewNode) -> Option<&str> {
    match node {
        ViewNode::Element {
            component, props, ..
        } if *component == SLOT_OUTLET => slot_assignment(props),
        _ => None,
    }
}

/// Whether this node is a slot outlet, named or not (spec §2.12).
///
/// The single predicate for "this node is an outlet". [`project_slots`]
/// substitutes every node it answers `true` for; the reconciler refuses to
/// materialize one, so a node it answers `true` for never becomes retained.
/// Both must mean the same thing by construction, which is why there is one of
/// these rather than a `component == SLOT_OUTLET` test in each.
#[must_use]
pub fn is_outlet(node: &ViewNode) -> bool {
    matches!(node, ViewNode::Element { component, .. } if *component == SLOT_OUTLET)
}

/// Whether `nodes`, or anything beneath them, contains an outlet (spec §2.12).
///
/// [`project_slots`]'s early-out, and the runtime's stray-outlet check: an
/// outlet in a view that is nobody's render output will never be projected, and
/// the runtime reports one rather than letting it pass silently.
#[must_use]
pub fn has_outlet(nodes: &[ViewNode]) -> bool {
    nodes.iter().any(|node| match node {
        ViewNode::Element { children, .. } => is_outlet(node) || has_outlet(children.nodes()),
        ViewNode::Text { .. } => false,
    })
}

/// Projects `content` — a component host's authored children — into the outlets
/// declared by `output`, the component's own render (spec §2.12).
///
/// Each outlet is replaced, in place and in order, by every node of `content`
/// whose `slot="…"` names it. An outlet nothing names is replaced by nothing.
/// A child assigned to a slot the output declares no outlet for reaches no
/// position and does not appear — the compile-time rule is what refuses a
/// `slot="…"` naming an undeclared slot, and it has already run.
///
/// `output` is returned untouched, without walking or rebuilding it, when it
/// declares no outlet — which is every component that takes no children.
#[must_use]
pub fn project_slots(output: View, content: &ViewFragment) -> View {
    if !has_outlet(output.root().nodes()) {
        return output;
    }
    View::from_fragment(project_fragment(output.into_fragment(), content))
}

/// [`project_slots`] over one fragment.
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

/// [`project_slots`] into one node's children. Text has none.
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

/// Whether `node` is assigned to the slot called `name`.
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
