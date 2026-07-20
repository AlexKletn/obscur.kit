//! Resolves computed styles across the retained tree: [`StyleCoordinator`].

use std::collections::{HashMap, HashSet};

use obscur_geometry::Size;
use obscur_identity::NodeId;
use obscur_style::computed::{ChMetrics, ComputedStyle, ResolutionContext, StyleResolver};
use obscur_style::invalidation::StyleInvalidation;
use obscur_tree::tree::RetainedTree;

use super::StyleInputs;

/// The computed style for every node the coordinator resolved, keyed by
/// [`NodeId`].
pub type ComputedStyles = HashMap<NodeId, ComputedStyle>;

/// What [`StyleCoordinator::resolve_incremental`] did, so the frame pipeline can
/// classify the phases downstream of it (spec §3.4 phase 5, §4.3).
#[derive(Debug, Clone, Default)]
pub struct IncrementalRestyle {
    /// How many nodes were re-resolved — the frame's real style cost, and the
    /// number the `style.resolved` counter reports. Not the size of the resulting
    /// map, and not `previous.len()` either.
    pub resolved: usize,
    /// The style each re-resolved node had **before**, for those that had one.
    ///
    /// Together with [`introduced`](Self::introduced) this is the complete answer to
    /// "where can the map have changed?": a node absent from both was not
    /// re-resolved, so it provably still holds the style it did last frame.
    pub previous: ComputedStyles,
    /// Whether any re-resolved node had **no** retained style, i.e. the walk added
    /// an entry rather than replacing one.
    ///
    /// Such a node has no "before" to compare against, so it is not in `previous`
    /// and a consumer must not conclude anything from its absence there. Always
    /// `false` when the caller's precondition holds (the retained map covers the
    /// tree); it exists so that a caller which is wrong about that degrades to
    /// conservative rather than to silently skipping work.
    pub introduced: bool,
}

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
///
/// The same parent-first order is what makes `rem` resolvable at computed-value
/// time: the root **element** is resolved first (against the initial `font-size`,
/// which is what `rem` means *on* the root), and its absolute `font-size` becomes
/// the `rem` base every descendant is resolved against — see
/// [`ResolutionContext`].
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

    /// Attaches `metrics` to `ctx` when the caller supplied any.
    ///
    /// Both walks rebuild their context on reaching the root element (to swap in
    /// the real `rem` base), and a rebuild that forgot the port would silently
    /// un-resolve `ch` for the whole document *below the root* while the root
    /// itself stayed correct — a bug that hides in exactly the tree shape tests
    /// use. Funnelling every construction through here is what stops that.
    const fn attach<'a>(
        ctx: ResolutionContext<'a>,
        metrics: Option<&'a dyn ChMetrics>,
    ) -> ResolutionContext<'a> {
        match metrics {
            Some(metrics) => ctx.with_metrics(metrics),
            None => ctx,
        }
    }

    /// Resolves the [`ComputedStyle`] of every node reachable from the tree root,
    /// inheriting down from each node's parent (spec §4.14).
    ///
    /// Returns an empty map when the tree has no root. Otherwise the tree is
    /// walked parent-first and each node is resolved against its cascaded
    /// [`StyleInputs`] and its parent's already-computed style.
    ///
    /// `viewport` supplies the `vw` / `vh` bases every computed `<length>` is
    /// absolutized against (spec §4.5).
    ///
    /// `metrics` supplies the `ch` base by measuring the `0` glyph. It is optional
    /// only because a caller may not own a shaper; one that does should pass it, or
    /// every `ch` on the tree stays symbolic and reaches paint as `0` — see
    /// [`ResolutionContext::with_metrics`].
    #[must_use]
    pub fn resolve(
        &self,
        tree: &RetainedTree,
        inputs: &StyleInputs,
        viewport: Size,
        metrics: Option<&dyn ChMetrics>,
    ) -> ComputedStyles {
        let Some(root) = tree.root() else {
            return ComputedStyles::new();
        };
        let order = Self::preorder(root, |id| {
            tree.get(id)
                .map_or_else(Vec::new, |node| node.child_nodes().to_vec())
        });
        let root_element = order
            .iter()
            .map(|&(node, _)| node)
            .find(|&node| tree.get(node).is_some_and(|n| n.kind().is_element()));
        self.resolve_in_order(&order, inputs, viewport, root_element, metrics)
    }

    /// Re-resolves **only** the nodes `dirty` records, updating `styles` in place
    /// (spec §3.4 phase 5, §4.3).
    ///
    /// This is the same walk as [`resolve`](Self::resolve) — the same parent-first
    /// order, the same threaded parent style, the same `rem` base read off the root
    /// element — with the resolve step gated on the dirty set: a node the set does
    /// not record keeps the style it already had instead of recomputing an identical
    /// one. Sharing the walk is what makes the two paths agree by construction rather
    /// than by coincidence.
    ///
    /// # What the caller owes
    ///
    /// `styles` must be what this same walk produced over this same tree, and
    /// `dirty` must record **every** node whose resolved style can differ — see
    /// [`RestyleScope`](crate::invalidation::RestyleScope), which derives that set
    /// from the selector engine's navigation surface. A node missing from `dirty`
    /// is not checked; it is *assumed* unchanged. Since the marked set is
    /// downward-closed, inheritance needs no separate pass: a node whose parent
    /// changed is in the set with it.
    #[must_use]
    pub fn resolve_incremental(
        &self,
        tree: &RetainedTree,
        inputs: &StyleInputs,
        viewport: Size,
        dirty: &StyleInvalidation,
        styles: &mut ComputedStyles,
        metrics: Option<&dyn ChMetrics>,
    ) -> IncrementalRestyle {
        let mut out = IncrementalRestyle::default();
        let Some(root) = tree.root() else {
            return out;
        };
        let order = Self::preorder(root, |id| {
            tree.get(id)
                .map_or_else(Vec::new, |node| node.child_nodes().to_vec())
        });
        let root_element = order
            .iter()
            .map(|&(node, _)| node)
            .find(|&node| tree.get(node).is_some_and(|n| n.kind().is_element()));

        let mut ctx = Self::attach(
            ResolutionContext::for_root(viewport.width, viewport.height),
            metrics,
        );
        for &(node, parent) in &order {
            // The `contains_key` arm is a hole guard, not part of the model: the
            // caller's precondition says the retained map covers this tree, and if it
            // somehow does not, resolving the node is better than leaving it styleless
            // — but the caller must then not treat unlisted nodes as unchanged, which
            // is what `introduced` tells it.
            if dirty.is_invalidated(node) || !styles.contains_key(&node) {
                let parent_style = parent.and_then(|p| styles.get(&p));
                let computed = self.resolver.resolve(
                    inputs.cascaded_for(node),
                    inputs.custom_for(node),
                    parent_style,
                    ctx,
                );
                out.resolved += 1;
                match styles.insert(node, computed) {
                    Some(previous) => {
                        out.previous.insert(node, previous);
                    }
                    None => out.introduced = true,
                }
            }
            // Read the `rem` base back off whatever style the root element ended
            // with — freshly resolved or retained — so it is the font-size the root
            // actually paints at either way, exactly as in `resolve_in_order`.
            if Some(node) == root_element
                && let Some(style) = styles.get(&node)
            {
                ctx = Self::attach(
                    ResolutionContext::new(
                        StyleResolver::font_size_of(style),
                        viewport.width,
                        viewport.height,
                    ),
                    metrics,
                );
            }
        }
        out
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
    ///
    /// `root_element` is the node `rem` resolves against: the topmost **element**
    /// of the walk, which is what `:root` matches. It is not necessarily
    /// `order[0]` — [`RetainedTree::root`] is a synthetic non-element host node
    /// whose first element child is the document's actual root element.
    ///
    /// Because the walk is parent-first, that element is reached before anything
    /// that could refer to it, so its computed `font-size` is in hand in time to
    /// become the `rem` base. It is read back off the resolved style rather than
    /// recomputed, so `rem` cannot drift from the `font-size` the root actually
    /// painted at. Until it is reached (i.e. for the synthetic host above it),
    /// `rem` means the initial `font-size` — the same answer `rem` gets *on* the
    /// root element itself.
    fn resolve_in_order(
        &self,
        order: &[(NodeId, Option<NodeId>)],
        inputs: &StyleInputs,
        viewport: Size,
        root_element: Option<NodeId>,
        metrics: Option<&dyn ChMetrics>,
    ) -> ComputedStyles {
        let mut out: ComputedStyles = HashMap::with_capacity(order.len());
        let mut ctx = Self::attach(
            ResolutionContext::for_root(viewport.width, viewport.height),
            metrics,
        );
        for &(node, parent) in order {
            let parent_style = parent.and_then(|p| out.get(&p));
            let computed = self.resolver.resolve(
                inputs.cascaded_for(node),
                inputs.custom_for(node),
                parent_style,
                ctx,
            );
            if Some(node) == root_element {
                ctx = Self::attach(
                    ResolutionContext::new(
                        StyleResolver::font_size_of(&computed),
                        viewport.width,
                        viewport.height,
                    ),
                    metrics,
                );
            }
            out.insert(node, computed);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A viewport for tests that are about the walk, not about `vw`/`vh`.
    fn viewport() -> Size {
        Size::new(800.0, 600.0)
    }
    use obscur_style::cascade::ResolvedDeclarations;
    use obscur_style::computed::ComputedValue;
    use obscur_style::property::{PropertyId, PropertyValue, StyleColor, layout_keyword};

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
        assert!(
            coordinator
                .resolve(&tree, &inputs, viewport(), None)
                .is_empty()
        );
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
        let styles = coordinator.resolve_in_order(&order, &inputs, viewport(), Some(root), None);

        assert_eq!(color_of(&styles, root), Some(red));
        // Proves parent-first inheritance: descendants inherit the root's color.
        assert_eq!(color_of(&styles, child), Some(red));
        assert_eq!(color_of(&styles, grandchild), Some(red));
    }

    #[test]
    fn relative_font_weight_resolves_top_down_over_the_tree() {
        // root font-weight:700; child A `bolder` -> 900; child B (none) inherits 700.
        let (root, a, b) = (node(1), node(2), node(3));
        let order = vec![(root, None), (a, Some(root)), (b, Some(root))];

        let mut root_decls = ResolvedDeclarations::new();
        root_decls.insert(PropertyId::FontWeight, PropertyValue::Number(700.0));
        let mut a_decls = ResolvedDeclarations::new();
        a_decls.insert(
            PropertyId::FontWeight,
            PropertyValue::Keyword(layout_keyword::FONT_WEIGHT_BOLDER),
        );

        let mut inputs = StyleInputs::new();
        inputs.set_cascaded(root, root_decls);
        inputs.set_cascaded(a, a_decls);
        // Child B has no cascaded font-weight of its own.

        let styles =
            StyleCoordinator::new().resolve_in_order(&order, &inputs, viewport(), Some(root), None);
        let weight = |id| match styles
            .get(&id)
            .and_then(|s| s.get(PropertyId::FontWeight))
            .map(ComputedValue::value)
        {
            Some(PropertyValue::Number(n)) => Some(*n),
            _ => None,
        };
        // `bolder` resolves against the parent's computed 700 -> 900.
        assert_eq!(weight(a), Some(900.0));
        // No authored weight -> inherits the parent's 700.
        assert_eq!(weight(b), Some(700.0));
    }

    #[test]
    fn node_without_cascaded_input_still_gets_a_style() {
        let (root, child) = (node(1), node(2));
        let order = vec![(root, None), (child, Some(root))];

        // No inputs at all for any node.
        let inputs = StyleInputs::new();
        let coordinator = StyleCoordinator::new();
        let styles = coordinator.resolve_in_order(&order, &inputs, viewport(), Some(root), None);

        // Both nodes are resolved; the child gets defaults/inheritance.
        assert!(styles.contains_key(&child));
        // `opacity` is not inherited and defaults to its initial value 1.0.
        assert_eq!(
            styles.get(&child).and_then(ComputedStyle::opacity),
            Some(1.0)
        );
    }
}
