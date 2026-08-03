#![doc = include_str!("focus_order.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_interaction::focus::{FocusNode, FocusScope, TabOrder};
use obscur_tree::tree::RetainedTree;

use super::{SemanticTreeBuilder, StructuralStates};

#[doc = include_str!("focus_order.doc/FocusOrder.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FocusOrder {
    #[doc = include_str!("focus_order.doc/order.md")]
    order: TabOrder,
    #[doc = include_str!("focus_order.doc/trapped.md")]
    trapped: bool,
}

impl FocusOrder {
    #[doc = include_str!("focus_order.doc/of.md")]
    pub(crate) fn of(tree: &RetainedTree, scope: Option<&FocusScope>) -> Self {
        let mut nodes = Vec::new();
        if let Some(root) = tree.root() {
            let states = StructuralStates::of(tree);
            collect(tree, &states, root, scope, &mut nodes);
        }
        Self {
            order: TabOrder::compute(&nodes),
            trapped: scope.is_some(),
        }
    }

    #[doc = include_str!("focus_order.doc/step.md")]
    pub(crate) fn step(&self, current: Option<NodeId>, backwards: bool) -> Option<NodeId> {
        // Wrapping is unconditional, and that is the same decision either way:
        // an untrapped sequence wraps because this runtime owns one surface and
        // there is nowhere else for focus to go — no browser chrome, no next
        // window in a tab ring — so refusing to wrap would strand the user at
        // the last control. A trapped one wraps because that is what a trap IS.
        // The distinction is kept in `trapped` anyway, because the day a host
        // can pass focus onward the two answers stop agreeing.
        if backwards {
            self.order.previous_before(current, true)
        } else {
            self.order.next_after(current, true)
        }
    }

    #[doc = include_str!("focus_order.doc/len.md")]
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.order.len()
    }

    #[doc = include_str!("focus_order.doc/is_trapped.md")]
    #[cfg(test)]
    pub(crate) const fn is_trapped(&self) -> bool {
        self.trapped
    }
}

#[doc = include_str!("focus_order.doc/collect.md")]
fn collect(
    tree: &RetainedTree,
    states: &StructuralStates<'_>,
    node: NodeId,
    scope: Option<&FocusScope>,
    out: &mut Vec<FocusNode>,
) {
    if let Some(retained) = tree.get(node) {
        if is_tabbable(tree, states, node, scope) {
            out.push(FocusNode::new(node).focusable(true));
        }
        for child in retained.child_nodes().to_vec() {
            collect(tree, states, child, scope, out);
        }
    }
}

#[doc = include_str!("focus_order.doc/is_tabbable.md")]
fn is_tabbable(
    tree: &RetainedTree,
    states: &StructuralStates<'_>,
    node: NodeId,
    scope: Option<&FocusScope>,
) -> bool {
    let Some(retained) = tree.get(node) else {
        return false;
    };
    // The SAME answer the accessibility projection publishes, deliberately: a
    // control a screen reader is told it can focus and a control Tab refuses to
    // reach would be two truths about one node, and the one the user hits is
    // whichever they happen to be driving.
    if !SemanticTreeBuilder::role_of(retained).is_interactive() {
        return false;
    }
    if states.is_within_disabled(node) {
        return false;
    }
    scope.is_none_or(|scope| scope.contains(node))
}

#[cfg(test)]
mod tests {
    use super::FocusOrder;
    use obscur_identity::{ComponentId, NodeId, RuntimeId};
    use obscur_interaction::focus::{FocusScope, FocusScopeId};
    use obscur_view::view::{
        DISABLED, LOCAL_NAME, View, ViewFragment, ViewNode, ViewProperty, ViewValue,
    };

    use crate::UiRuntime;

    const PANEL: ComponentId = ComponentId::from_raw(0x00C0_0001);

    /// An element carrying `local_name`, which is what the role — and so
    /// tabbability — is derived from.
    fn named(name: &str) -> ViewNode {
        ViewNode::element(ComponentId::from_raw(0x00D0_0001))
            .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::Text(name.into())))
    }

    fn committed(nodes: Vec<ViewNode>) -> (UiRuntime, Vec<NodeId>) {
        let mut rt = UiRuntime::new(
            RuntimeId::from_raw(1),
            obscur_geometry::Size::new(200.0, 200.0),
        );
        let view =
            View::single(ViewNode::element(PANEL).with_children(ViewFragment::from_nodes(nodes)));
        let _ = rt.render_now(&view, &[]);
        let root = rt.tree().root().expect("a root");
        let panel = rt.tree().get(root).expect("live").child_nodes()[0];
        let kids = rt.tree().get(panel).expect("live").child_nodes().to_vec();
        (rt, kids)
    }

    #[test]
    fn only_the_interactive_roles_are_in_the_sequence() {
        // A `div` between two controls is not a stop. This is the whole reason
        // the sequence is role-derived: a press focuses whatever it hit,
        // including a text leaf, and a Tab ring built from that would walk the
        // user through every box on the screen.
        let (rt, kids) = committed(vec![named("button"), named("div"), named("input")]);
        let order = FocusOrder::of(rt.tree(), None);
        assert_eq!(order.len(), 2);
        assert_eq!(order.step(None, false), Some(kids[0]));
        assert_eq!(order.step(Some(kids[0]), false), Some(kids[2]));
    }

    #[test]
    fn a_disabled_control_is_skipped() {
        let disabled = named("button").with_prop(ViewProperty::new(DISABLED, ViewValue::Null));
        let (rt, kids) = committed(vec![named("button"), disabled, named("button")]);
        let order = FocusOrder::of(rt.tree(), None);
        assert_eq!(order.len(), 2, "the disabled control is not a stop");
        assert_eq!(order.step(Some(kids[0]), false), Some(kids[2]));
    }

    #[test]
    fn the_sequence_wraps_in_both_directions() {
        let (rt, kids) = committed(vec![named("button"), named("input")]);
        let order = FocusOrder::of(rt.tree(), None);
        assert_eq!(order.step(Some(kids[1]), false), Some(kids[0]), "forward");
        assert_eq!(order.step(Some(kids[0]), true), Some(kids[1]), "backward");
    }

    #[test]
    fn focus_on_a_node_outside_the_sequence_starts_from_an_end() {
        // The case a pointer creates constantly: a press focuses a plain box, so
        // `current` is not in the ring at all. Tab must start somewhere sensible
        // rather than answer nothing.
        let (rt, kids) = committed(vec![named("div"), named("button")]);
        let order = FocusOrder::of(rt.tree(), None);
        assert_eq!(order.step(Some(kids[0]), false), Some(kids[1]));
        assert_eq!(order.step(Some(kids[0]), true), Some(kids[1]));
    }

    #[test]
    fn a_scope_traps_the_sequence_to_its_own_nodes() {
        let (rt, kids) = committed(vec![named("button"), named("input"), named("button")]);
        let scope = FocusScope::new(FocusScopeId::from_raw(1))
            .trapping(true)
            .with_member(kids[1])
            .with_member(kids[2]);
        let order = FocusOrder::of(rt.tree(), Some(&scope));
        assert!(order.is_trapped());
        assert_eq!(
            order.len(),
            2,
            "the control outside the scope is not a stop"
        );
        assert_eq!(
            order.step(Some(kids[2]), false),
            Some(kids[1]),
            "and the wrap stays inside it"
        );
    }

    #[test]
    fn an_empty_sequence_answers_nothing_rather_than_panicking() {
        let (rt, _kids) = committed(vec![named("div")]);
        let order = FocusOrder::of(rt.tree(), None);
        assert_eq!(order.len(), 0);
        assert_eq!(order.step(None, false), None);
    }
}
