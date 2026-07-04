//! Resolves computed styles across the retained tree: [`StyleCoordinator`].

use std::collections::{HashMap, HashSet};

use obscur_identity::NodeId;
use obscur_style::computed::{ComputedStyle, StyleResolver};
use obscur_tree::tree::RetainedTree;

use super::StyleInputs;

/// The computed style for every node the coordinator resolved, keyed by
/// [`NodeId`].
pub type ComputedStyles = HashMap<NodeId, ComputedStyle>;

/// Orchestrates computed-style resolution over a [`RetainedTree`].
///
/// The coordinator is **orchestration only** (spec §3.10): the computed-value
/// algorithm and inheritance live entirely in `obscur-style`'s [`StyleResolver`].
/// The coordinator's own responsibility is to walk the tree **parent-first**
/// (pre-order) and thread each node's already-computed parent style into the
/// resolver, so that when a node is resolved its parent's [`ComputedStyle`] is
/// already available for inheritance (spec §4.14).
///
/// The walk is cycle-safe and panic-free: a malformed or cyclic child link never
/// causes a loop, and a node that is unreachable from the root is simply not
/// resolved.
#[derive(Debug, Clone, Default)]
pub struct StyleCoordinator {
    /// The stateless resolver that computes each node's style.
    resolver: StyleResolver,
}

impl StyleCoordinator {
    /// Creates a coordinator with a fresh resolver.
    #[must_use]
    pub fn new() -> Self {
        Self {
            resolver: StyleResolver::new(),
        }
    }

    /// Resolves the [`ComputedStyle`] of every node reachable from the tree root,
    /// inheriting down from each node's parent (spec §4.14).
    ///
    /// Returns an empty map when the tree has no root. Otherwise the tree is
    /// walked parent-first and each node is resolved against its cascaded
    /// [`StyleInputs`] and its parent's already-computed style.
    #[must_use]
    pub fn resolve(&self, tree: &RetainedTree, inputs: &StyleInputs) -> ComputedStyles {
        let Some(root) = tree.root() else {
            return ComputedStyles::new();
        };
        let order = Self::preorder(root, |id| {
            tree.get(id)
                .map_or_else(Vec::new, |node| node.children().to_vec())
        });
        self.resolve_in_order(&order, inputs)
    }

    /// Computes a parent-first (pre-order) traversal from `root`, pairing each
    /// visited node with its parent id (the root's parent is [`None`]).
    ///
    /// `children_of` yields the child ids of a node. The traversal is cycle-safe:
    /// a visited set guarantees each node is emitted at most once, so cyclic or
    /// diamond child links terminate rather than loop. Children are emitted in
    /// their natural left-to-right order.
    fn preorder<F>(root: NodeId, children_of: F) -> Vec<(NodeId, Option<NodeId>)>
    where
        F: Fn(NodeId) -> Vec<NodeId>,
    {
        let mut order: Vec<(NodeId, Option<NodeId>)> = Vec::new();
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut stack: Vec<(NodeId, Option<NodeId>)> = vec![(root, None)];

        while let Some((node, parent)) = stack.pop() {
            if !visited.insert(node) {
                // Already emitted (cycle or shared child): do not revisit.
                continue;
            }
            order.push((node, parent));
            // Push children reversed so the leftmost child is popped first,
            // preserving left-to-right pre-order.
            for &child in children_of(node).iter().rev() {
                if !visited.contains(&child) {
                    stack.push((child, Some(node)));
                }
            }
        }

        order
    }

    /// Resolves every node in a parent-first `order`, threading each node's
    /// already-computed parent style into the resolver.
    ///
    /// Because `order` is parent-first, a node's parent is always present in the
    /// accumulating map by the time the node is resolved, so inheritance flows
    /// down correctly. A node with no cascaded inputs still gets a computed style
    /// from the resolver's defaults and inheritance.
    fn resolve_in_order(
        &self,
        order: &[(NodeId, Option<NodeId>)],
        inputs: &StyleInputs,
    ) -> ComputedStyles {
        let mut out: ComputedStyles = HashMap::with_capacity(order.len());
        for &(node, parent) in order {
            let parent_style = parent.and_then(|p| out.get(&p));
            let computed = self.resolver.resolve(
                inputs.cascaded_for(node),
                inputs.custom_for(node),
                parent_style,
            );
            out.insert(node, computed);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_style::cascade::ResolvedDeclarations;
    use obscur_style::computed::ComputedValue;
    use obscur_style::property::{PropertyId, PropertyValue, StyleColor};

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    fn color_of(styles: &ComputedStyles, id: NodeId) -> Option<StyleColor> {
        styles
            .get(&id)
            .and_then(|style| style.get(PropertyId::Color))
            .and_then(ComputedValue::as_color)
    }

    #[test]
    fn empty_tree_resolves_to_empty_map() {
        let coordinator = StyleCoordinator::new();
        let tree = RetainedTree::new();
        let inputs = StyleInputs::new();
        assert!(coordinator.resolve(&tree, &inputs).is_empty());
    }

    #[test]
    fn preorder_is_parent_first() {
        // root -> child -> grandchild (a chain).
        let (root, child, grandchild) = (node(1), node(2), node(3));
        let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        children.insert(root, vec![child]);
        children.insert(child, vec![grandchild]);

        let order =
            StyleCoordinator::preorder(root, |id| children.get(&id).cloned().unwrap_or_default());

        assert_eq!(
            order,
            vec![(root, None), (child, Some(root)), (grandchild, Some(child)),]
        );
    }

    #[test]
    fn preorder_visits_siblings_left_to_right_parent_before_children() {
        // root -> [a, b]; a -> [a1].
        let (root, a, b, a1) = (node(1), node(2), node(3), node(4));
        let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        children.insert(root, vec![a, b]);
        children.insert(a, vec![a1]);

        let order =
            StyleCoordinator::preorder(root, |id| children.get(&id).cloned().unwrap_or_default());

        // Every parent appears before each of its children.
        let position = |target: NodeId| order.iter().position(|&(n, _)| n == target);
        assert!(position(root) < position(a));
        assert!(position(a) < position(a1));
        assert!(position(root) < position(b));
        // Left-to-right: a (and its subtree) before b.
        assert!(position(a) < position(b));
        assert_eq!(order.len(), 4);
    }

    #[test]
    fn preorder_is_cycle_safe() {
        // root -> child -> root (a cycle) and child -> root again via diamond.
        let (root, child) = (node(1), node(2));
        let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        children.insert(root, vec![child]);
        children.insert(child, vec![root]);

        let order =
            StyleCoordinator::preorder(root, |id| children.get(&id).cloned().unwrap_or_default());

        // Terminates and emits each node exactly once, parent-first.
        assert_eq!(order, vec![(root, None), (child, Some(root))]);
    }

    #[test]
    fn inheritance_flows_down_parent_first() {
        // root -> child -> grandchild; `color` inherits by default.
        let (root, child, grandchild) = (node(1), node(2), node(3));
        let order = vec![(root, None), (child, Some(root)), (grandchild, Some(child))];

        let red = StyleColor::rgba(1.0, 0.0, 0.0, 1.0);
        let mut root_decls = ResolvedDeclarations::new();
        root_decls.insert(PropertyId::Color, PropertyValue::Color(red));

        let mut inputs = StyleInputs::new();
        inputs.set_cascaded(root, root_decls);
        // child and grandchild have no cascaded inputs of their own.

        let coordinator = StyleCoordinator::new();
        let styles = coordinator.resolve_in_order(&order, &inputs);

        assert_eq!(color_of(&styles, root), Some(red));
        // Proves parent-first inheritance: descendants inherit the root's color.
        assert_eq!(color_of(&styles, child), Some(red));
        assert_eq!(color_of(&styles, grandchild), Some(red));
    }

    #[test]
    fn node_without_cascaded_input_still_gets_a_style() {
        let (root, child) = (node(1), node(2));
        let order = vec![(root, None), (child, Some(root))];

        // No inputs at all for any node.
        let inputs = StyleInputs::new();
        let coordinator = StyleCoordinator::new();
        let styles = coordinator.resolve_in_order(&order, &inputs);

        // Both nodes are resolved; the child gets defaults/inheritance.
        assert!(styles.contains_key(&child));
        // `opacity` is not inherited and defaults to its initial value 1.0.
        assert_eq!(
            styles.get(&child).and_then(ComputedStyle::opacity),
            Some(1.0)
        );
    }
}
