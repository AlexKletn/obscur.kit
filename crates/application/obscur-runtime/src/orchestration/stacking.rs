//! Stacking (paint) order for positioned elements: a bounded, flat CSS
//! `z-index` model.
//!
//! Full CSS 2.1 stacking contexts (Appendix E) are intricate — nested contexts,
//! the `auto`/non-`auto` split, opacity/transform-induced contexts, and the
//! seven-layer within-context ordering. This module implements a **flat**,
//! tractable subset that is correct for the common case of a handful of
//! positioned elements with `z-index` overlapping:
//!
//! * Every node is assigned an **effective stacking z** (an `i32`). A node's z is
//!   its own `z-index` when it is **positioned** (`position != static`) and its
//!   `z-index` is a numeric `<integer>`; otherwise it inherits its parent's
//!   effective z (root = `0`). `z-index: auto` therefore stacks *with* the parent
//!   (effective z of the nearest positioned-with-integer ancestor), so a
//!   positioned subtree with a `z-index` moves together in z — its background and
//!   its descendants share one stacking level.
//! * The scene collects each node's own paint (its background / text — not its
//!   descendants') as a *paint unit* keyed `(z, tree_index)` and STABLE-SORTS the
//!   units by `(z ascending, tree_index ascending)`. Higher z paints last (on
//!   top); ties keep document (pre-order) order.
//!
//! **Not modelled** (documented simplifications): nested stacking-context
//! isolation (a child's high z can escape its parent's context, unlike real
//! CSS), the `auto`-establishes-no-context nuance, and opacity/transform/filter
//! induced stacking contexts. When no node authors a non-`auto` `z-index`, every
//! effective z is `0` and the stable sort by `tree_index` reproduces the exact
//! pre-order paint sequence — the regression guard.

use std::collections::{HashMap, HashSet};

use obscur_identity::NodeId;
use obscur_scene::command::PaintCommand;
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{layout_keyword, PropertyId, PropertyValue};
use obscur_tree::tree::RetainedTree;

use super::ComputedStyles;

/// One node's own paint commands, tagged with its stacking key.
///
/// The commands are exactly the node's own background / text (never its
/// descendants'), captured in emission order. The key `(z, tree_index)` orders
/// the unit against every other unit: `z` is the node's effective stacking level
/// and `tree_index` its pre-order document position (the tie-breaker).
pub(super) struct PaintUnit {
    /// The node's effective stacking z (see [`effective_stacking_z`]).
    pub z: i32,
    /// The node's pre-order document position, tie-breaking equal-z units so ties
    /// keep tree order.
    pub tree_index: usize,
    /// The node's own paint commands, in emission order.
    pub commands: Vec<PaintCommand>,
}

/// Computes each node's **effective stacking z** for the whole tree.
///
/// Walks pre-order from the root (cycle-safe via a visited set); the root's
/// effective z is `0`. Each node's effective z is its own `z-index` when it is
/// positioned with a numeric `<integer>` `z-index` (see [`own_stacking_z`]),
/// otherwise it inherits its parent's effective z. This makes a positioned
/// subtree with a `z-index` stack together, and leaves every z at `0` when no
/// `z-index` is authored.
#[must_use]
pub(super) fn effective_stacking_z(
    tree: &RetainedTree,
    computed: &ComputedStyles,
) -> HashMap<NodeId, i32> {
    let mut zmap: HashMap<NodeId, i32> = HashMap::new();
    let Some(root) = tree.root() else {
        return zmap;
    };
    let mut visited: HashSet<NodeId> = HashSet::new();
    // Each stack entry carries the effective z inherited from its parent chain.
    let mut stack: Vec<(NodeId, i32)> = vec![(root, 0)];
    while let Some((node, inherited)) = stack.pop() {
        if !visited.insert(node) {
            continue;
        }
        let z = own_stacking_z(computed, node).unwrap_or(inherited);
        zmap.insert(node, z);
        if let Some(retained) = tree.get(node) {
            // Children inherit this node's effective z (reversed push keeps the
            // left-to-right pre-order the emission walk relies on).
            for &child in retained.children().iter().rev() {
                if !visited.contains(&child) {
                    stack.push((child, z));
                }
            }
        }
    }
    zmap
}

/// Reads a node's **own** stacking z: its numeric `z-index`, but only when the
/// node is positioned (`position != static`).
///
/// Returns [`None`] for a static node, a positioned node whose `z-index` is
/// `auto` (or absent), or any node without a computed style — in every case the
/// node inherits its parent's effective z rather than establishing its own level.
fn own_stacking_z(computed: &ComputedStyles, node: NodeId) -> Option<i32> {
    let style = computed.get(&node)?;
    if !is_positioned(style) {
        return None;
    }
    match style.get(PropertyId::ZIndex).map(ComputedValue::value) {
        Some(&PropertyValue::Number(n)) => Some(z_index_to_i32(n)),
        _ => None,
    }
}

/// Returns whether a computed style is **positioned** (`position` is `relative`,
/// `absolute`, or `fixed`); a `static` or absent `position` is not positioned and
/// so ignores `z-index` entirely (CSS §5.1).
fn is_positioned(style: &ComputedStyle) -> bool {
    match style.get(PropertyId::Position).map(ComputedValue::value) {
        Some(&PropertyValue::Keyword(sym)) => {
            sym == layout_keyword::POSITION_RELATIVE
                || sym == layout_keyword::POSITION_ABSOLUTE
                || sym == layout_keyword::POSITION_FIXED
        }
        _ => false,
    }
}

/// Narrows the computed `z-index` number (an already-integral `<integer>`, stored
/// as `f32`) into the `i32` stacking level.
///
/// The CSS lowering only admits integral numbers, so truncation is exact for the
/// modelled range; a pathological out-of-range value saturates rather than
/// wrapping.
#[allow(clippy::cast_possible_truncation)]
fn z_index_to_i32(n: f32) -> i32 {
    // Round to the nearest integer (the value is already integral) then clamp to
    // the `i32` range so an extreme authored value can't wrap the ordering key.
    let rounded = n.round();
    if rounded >= i32::MAX as f32 {
        i32::MAX
    } else if rounded <= i32::MIN as f32 {
        i32::MIN
    } else {
        rounded as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;
    use obscur_tree::node::{LogicalParent, NodeKind};

    fn positioned_with_z(z: f32) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::Position,
            ComputedValue::keyword(layout_keyword::POSITION_ABSOLUTE),
        );
        style.set(PropertyId::ZIndex, ComputedValue::number(z));
        style
    }

    fn add_child(tree: &mut RetainedTree, parent: NodeId, raw: u64) -> NodeId {
        let id = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(raw),
        });
        if let Some(node) = tree.get_mut(parent) {
            node.push_child(id);
        }
        if let Some(node) = tree.get_mut(id) {
            node.set_logical_parent(LogicalParent::of(parent));
        }
        id
    }

    #[test]
    fn all_static_nodes_are_zero() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 1);
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, ComputedStyle::new());

        let zmap = effective_stacking_z(&tree, &computed);
        assert_eq!(zmap.get(&root), Some(&0));
        assert_eq!(zmap.get(&a), Some(&0));
    }

    #[test]
    fn positioned_node_uses_its_own_z() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 1);
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, positioned_with_z(5.0));

        let zmap = effective_stacking_z(&tree, &computed);
        assert_eq!(zmap.get(&a), Some(&5));
    }

    #[test]
    fn descendants_inherit_the_nearest_z_ancestor() {
        // root(0) -> a (positioned z=3) -> b (static) : b inherits a's z (3).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 1);
        let b = add_child(&mut tree, a, 2);
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, positioned_with_z(3.0));
        computed.insert(b, ComputedStyle::new());

        let zmap = effective_stacking_z(&tree, &computed);
        assert_eq!(zmap.get(&a), Some(&3));
        assert_eq!(zmap.get(&b), Some(&3), "descendant stacks with its z ancestor");
    }

    #[test]
    fn z_index_on_static_node_is_ignored() {
        // A `z-index` on a non-positioned node has no effect (stays 0).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 1);
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        let mut style = ComputedStyle::new();
        style.set(PropertyId::ZIndex, ComputedValue::number(9.0));
        computed.insert(a, style);

        let zmap = effective_stacking_z(&tree, &computed);
        assert_eq!(zmap.get(&a), Some(&0));
    }

    #[test]
    fn negative_z_is_preserved() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 1);
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, positioned_with_z(-2.0));

        let zmap = effective_stacking_z(&tree, &computed);
        assert_eq!(zmap.get(&a), Some(&-2));
    }
}
