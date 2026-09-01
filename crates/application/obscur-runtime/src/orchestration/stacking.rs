#![doc = include_str!("stacking.doc/_module.md")]

use std::collections::{HashMap, HashSet};

use obscur_identity::{FastMap, FastSet, NodeId};
use obscur_scene::command::PaintCommand;
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{PropertyId, PropertyValue, layout_keyword};
use obscur_tree::tree::RetainedTree;

use super::ComputedStyles;

#[doc = include_str!("stacking.doc/PaintUnit.md")]
pub(super) struct PaintUnit {
    #[doc = include_str!("stacking.doc/z.md")]
    pub z: i32,
    #[doc = include_str!("stacking.doc/layer.md")]
    pub layer: PaintLayer,
    #[doc = include_str!("stacking.doc/tree_index.md")]
    pub tree_index: usize,
    #[doc = include_str!("stacking.doc/order.md")]
    pub order: i64,
    #[doc = include_str!("stacking.doc/commands.md")]
    pub commands: Vec<PaintCommand>,
}

impl PaintUnit {
    #[doc = include_str!("stacking.doc/BLUR_PUSH_ORDER.md")]
    pub(super) const BLUR_PUSH_ORDER: i64 = -4;
    #[doc = include_str!("stacking.doc/TRANSFORM_PUSH_ORDER.md")]
    pub(super) const TRANSFORM_PUSH_ORDER: i64 = -3;
    #[doc = include_str!("stacking.doc/LAYER_PUSH_ORDER.md")]
    pub(super) const LAYER_PUSH_ORDER: i64 = -2;
    #[doc = include_str!("stacking.doc/CLIP_PUSH_ORDER.md")]
    pub(super) const CLIP_PUSH_ORDER: i64 = -1;
    #[doc = include_str!("stacking.doc/PAINT_ORDER.md")]
    pub(super) const PAINT_ORDER: i64 = 0;

    #[doc = include_str!("stacking.doc/SCROLL_TRANSFORM_PUSH_ORDER.md")]
    pub(super) const SCROLL_TRANSFORM_PUSH_ORDER: i64 = 1;

    #[doc = include_str!("stacking.doc/SCROLLBAR_ORDER.md")]
    pub(super) const SCROLLBAR_ORDER: i64 = i64::MAX;

    #[doc = include_str!("stacking.doc/OUTLINE_ORDER.md")]
    pub(super) const OUTLINE_ORDER: i64 = i64::MAX;

    #[doc = include_str!("stacking.doc/clip_pop_order.md")]
    #[must_use]
    pub(super) const fn clip_pop_order(depth: u32) -> i64 {
        i64::MAX - depth as i64
    }

    #[doc = include_str!("stacking.doc/layer_pop_order.md")]
    #[must_use]
    pub(super) const fn layer_pop_order(depth: u32) -> i64 {
        i64::MAX - depth as i64
    }

    #[doc = include_str!("stacking.doc/transform_pop_order.md")]
    #[must_use]
    pub(super) const fn transform_pop_order(depth: u32) -> i64 {
        i64::MAX - depth as i64
    }

    #[doc = include_str!("stacking.doc/blur_pop_order.md")]
    #[must_use]
    pub(super) const fn blur_pop_order(depth: u32) -> i64 {
        i64::MAX - depth as i64
    }
}

#[doc = include_str!("stacking.doc/PaintLayer.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum PaintLayer {
    #[doc = include_str!("stacking.doc/NonPositioned.md")]
    NonPositioned,
    #[doc = include_str!("stacking.doc/PositionedZeroOrAuto.md")]
    PositionedZeroOrAuto,
}

#[doc = include_str!("stacking.doc/EffectiveStacking.md")]
pub(crate) struct EffectiveStacking {
    #[doc = include_str!("stacking.doc/z.md")]
    pub(crate) z: FastMap<NodeId, i32>,
    #[doc = include_str!("stacking.doc/layer.md")]
    pub(crate) layer: FastMap<NodeId, PaintLayer>,
}

#[doc = include_str!("stacking.doc/effective_stacking.md")]
#[must_use]
pub(crate) fn effective_stacking(
    tree: &RetainedTree,
    computed: &ComputedStyles,
) -> EffectiveStacking {
    // One walk, two answers. These were two functions with byte-identical
    // traversals — same stack, same visited set, same inheritance rule — run back
    // to back on every composed frame and again on every pointer event, each
    // asking the same node for the same `position` and `z-index`. The two
    // quantities are read off one `ComputedStyle` lookup.
    let mut out = EffectiveStacking {
        z: HashMap::default(),
        layer: HashMap::default(),
    };
    let Some(root) = tree.root() else {
        return out;
    };
    let mut visited: FastSet<NodeId> = HashSet::default();
    let mut stack: Vec<(NodeId, i32, PaintLayer)> = vec![(root, 0, PaintLayer::NonPositioned)];
    while let Some((node, inherited_z, inherited_layer)) = stack.pop() {
        if !visited.insert(node) {
            continue;
        }
        let own = own_stacking(computed, node);
        let z = own.and_then(|(z, _)| z).unwrap_or(inherited_z);
        let layer = own.map_or(inherited_layer, |(_, layer)| layer);
        out.z.insert(node, z);
        out.layer.insert(node, layer);
        if let Some(retained) = tree.get(node) {
            // Children inherit this node's effective values (reversed push keeps
            // the left-to-right pre-order the emission walk relies on).
            for &child in retained.child_nodes().iter().rev() {
                if !visited.contains(&child) {
                    stack.push((child, z, layer));
                }
            }
        }
    }
    out
}

#[doc = include_str!("stacking.doc/own_stacking.md")]
fn own_stacking(computed: &ComputedStyles, node: NodeId) -> Option<(Option<i32>, PaintLayer)> {
    let style = computed.get(&node)?;
    if !is_positioned(style) {
        return None;
    }
    match style.get(PropertyId::ZIndex).map(ComputedValue::value) {
        Some(&PropertyValue::Number(n)) => {
            let z = z_index_to_i32(n);
            // A non-zero integer z-index is isolated by the primary `z` key, so
            // it needs no lift from the layer rank.
            let layer = if z == 0 {
                PaintLayer::PositionedZeroOrAuto
            } else {
                PaintLayer::NonPositioned
            };
            Some((Some(z), layer))
        }
        // `auto` (absent / non-numeric) or an explicit `0`: CSS 2.1 §9.9 layer 6,
        // and no own `z` — the nearest z-index ancestor's is inherited.
        _ => Some((None, PaintLayer::PositionedZeroOrAuto)),
    }
}

#[doc = include_str!("stacking.doc/is_positioned.md")]
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

#[doc = include_str!("stacking.doc/z_index_to_i32.md")]
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
    /// The two projections of [`effective_stacking`], so every assertion in
    /// this module still reads as the question it is asking. Both call the real
    /// walk — nothing here reimplements it.
    fn effective_stacking_z(
        tree: &RetainedTree,
        computed: &ComputedStyles,
    ) -> FastMap<NodeId, i32> {
        effective_stacking(tree, computed).z
    }

    fn effective_paint_layer(
        tree: &RetainedTree,
        computed: &ComputedStyles,
    ) -> FastMap<NodeId, PaintLayer> {
        effective_stacking(tree, computed).layer
    }

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
        let mut computed = ComputedStyles::default();
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
        let mut computed = ComputedStyles::default();
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
        let mut computed = ComputedStyles::default();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, positioned_with_z(3.0));
        computed.insert(b, ComputedStyle::new());

        let zmap = effective_stacking_z(&tree, &computed);
        assert_eq!(zmap.get(&a), Some(&3));
        assert_eq!(
            zmap.get(&b),
            Some(&3),
            "descendant stacks with its z ancestor"
        );
    }

    #[test]
    fn z_index_on_static_node_is_ignored() {
        // A `z-index` on a non-positioned node has no effect (stays 0).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 1);
        let mut computed = ComputedStyles::default();
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
        let mut computed = ComputedStyles::default();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, positioned_with_z(-2.0));

        let zmap = effective_stacking_z(&tree, &computed);
        assert_eq!(zmap.get(&a), Some(&-2));
    }

    // ---- §9.9 paint-layer rank (bug B2) ----------------------------------

    /// A bare positioned element (no `z-index`, i.e. `z-index: auto`).
    fn positioned_auto() -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::Position,
            ComputedValue::keyword(layout_keyword::POSITION_ABSOLUTE),
        );
        style
    }

    #[test]
    fn all_static_nodes_share_the_non_positioned_rank() {
        // The byte-identical regression guard: nothing positioned → every rank is
        // `NonPositioned`, so the layer key is uniform and the sort reduces to
        // `tree_index`.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 1);
        let b = add_child(&mut tree, a, 2);
        let mut computed = ComputedStyles::default();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, ComputedStyle::new());
        computed.insert(b, ComputedStyle::new());

        let layers = effective_paint_layer(&tree, &computed);
        assert_eq!(layers.get(&root), Some(&PaintLayer::NonPositioned));
        assert_eq!(layers.get(&a), Some(&PaintLayer::NonPositioned));
        assert_eq!(layers.get(&b), Some(&PaintLayer::NonPositioned));
    }

    #[test]
    fn positioned_z_auto_or_zero_takes_the_lift_rank() {
        // A positioned `z:auto` box and a positioned `z:0` box both land in §9.9
        // layer 6, ABOVE a non-positioned sibling in the same z bucket.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let auto = add_child(&mut tree, root, 1);
        let zero = add_child(&mut tree, root, 2);
        let static_block = add_child(&mut tree, root, 3);
        let mut computed = ComputedStyles::default();
        computed.insert(root, ComputedStyle::new());
        computed.insert(auto, positioned_auto());
        computed.insert(zero, positioned_with_z(0.0));
        computed.insert(static_block, ComputedStyle::new());

        let layers = effective_paint_layer(&tree, &computed);
        let zmap = effective_stacking_z(&tree, &computed);
        // All three share the z == 0 bucket …
        assert_eq!(zmap.get(&auto), Some(&0));
        assert_eq!(zmap.get(&zero), Some(&0));
        assert_eq!(zmap.get(&static_block), Some(&0));
        // … but the positioned pair lifts above the non-positioned block.
        assert_eq!(layers.get(&auto), Some(&PaintLayer::PositionedZeroOrAuto));
        assert_eq!(layers.get(&zero), Some(&PaintLayer::PositionedZeroOrAuto));
        assert_eq!(layers.get(&static_block), Some(&PaintLayer::NonPositioned));
        assert!(PaintLayer::NonPositioned < PaintLayer::PositionedZeroOrAuto);
    }

    #[test]
    fn positioned_lift_is_inherited_by_the_subtree() {
        // A positioned `z:auto` box's non-positioned descendants inherit the lift so
        // the whole positioned box rises together (ancestor still below its own
        // descendants by tree order).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let pos = add_child(&mut tree, root, 1);
        let child = add_child(&mut tree, pos, 2);
        let mut computed = ComputedStyles::default();
        computed.insert(root, ComputedStyle::new());
        computed.insert(pos, positioned_auto());
        computed.insert(child, ComputedStyle::new());

        let layers = effective_paint_layer(&tree, &computed);
        assert_eq!(layers.get(&pos), Some(&PaintLayer::PositionedZeroOrAuto));
        assert_eq!(
            layers.get(&child),
            Some(&PaintLayer::PositionedZeroOrAuto),
            "a positioned box's subtree inherits the layer-6 lift"
        );
    }

    #[test]
    fn positioned_integer_z_is_not_lifted() {
        // A positioned box with a non-zero integer z-index is isolated by the
        // primary `z` key, so it keeps the `NonPositioned` rank (it must not lift
        // above its own descendants, which share its z).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let up = add_child(&mut tree, root, 1);
        let down = add_child(&mut tree, root, 2);
        let mut computed = ComputedStyles::default();
        computed.insert(root, ComputedStyle::new());
        computed.insert(up, positioned_with_z(5.0));
        computed.insert(down, positioned_with_z(-2.0));

        let layers = effective_paint_layer(&tree, &computed);
        assert_eq!(layers.get(&up), Some(&PaintLayer::NonPositioned));
        assert_eq!(layers.get(&down), Some(&PaintLayer::NonPositioned));
    }

    #[test]
    fn sort_key_lifts_positioned_above_earlier_non_positioned() {
        // The end-to-end ordering the fix delivers: a positioned `z:auto` box (B)
        // that comes EARLIER in the tree than a non-positioned block (A) must paint
        // LAST (on top), even though its `tree_index` is smaller. Pre-fix the
        // `(z, tree_index)` sort kept B below A.
        let a_non_positioned = PaintUnit {
            z: 0,
            layer: PaintLayer::NonPositioned,
            tree_index: 2, // later in the tree
            order: PaintUnit::PAINT_ORDER,
            commands: Vec::new(),
        };
        let b_positioned = PaintUnit {
            z: 0,
            layer: PaintLayer::PositionedZeroOrAuto,
            tree_index: 1, // earlier in the tree
            order: PaintUnit::PAINT_ORDER,
            commands: Vec::new(),
        };
        let mut units = vec![b_positioned, a_non_positioned];
        units.sort_by_key(|u| (u.z, u.layer, u.tree_index, u.order));
        // A (non-positioned, later) paints first; B (positioned, earlier) paints
        // last — on top.
        assert_eq!(units[0].tree_index, 2, "non-positioned A paints first");
        assert_eq!(units[1].tree_index, 1, "positioned B paints last (on top)");
    }

    #[test]
    fn sort_key_keeps_non_positioned_in_tree_order() {
        // Regression guard at the sort level: equal (z, layer) units keep
        // `tree_index` (document) order — byte-identical to the pre-fix sequence.
        let mut units = vec![
            PaintUnit {
                z: 0,
                layer: PaintLayer::NonPositioned,
                tree_index: 3,
                order: PaintUnit::PAINT_ORDER,
                commands: Vec::new(),
            },
            PaintUnit {
                z: 0,
                layer: PaintLayer::NonPositioned,
                tree_index: 1,
                order: PaintUnit::PAINT_ORDER,
                commands: Vec::new(),
            },
            PaintUnit {
                z: 0,
                layer: PaintLayer::NonPositioned,
                tree_index: 2,
                order: PaintUnit::PAINT_ORDER,
                commands: Vec::new(),
            },
        ];
        units.sort_by_key(|u| (u.z, u.layer, u.tree_index, u.order));
        assert_eq!(
            units.iter().map(|u| u.tree_index).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn sort_key_negative_z_paints_below_non_positioned() {
        // A negative-z positioned box (C) sorts below a non-positioned block (A) via
        // the PRIMARY `z` key — the layer rank is not even consulted across z.
        let a_non_positioned = PaintUnit {
            z: 0,
            layer: PaintLayer::NonPositioned,
            tree_index: 1,
            order: PaintUnit::PAINT_ORDER,
            commands: Vec::new(),
        };
        let c_negative = PaintUnit {
            z: -2,
            layer: PaintLayer::NonPositioned,
            tree_index: 2,
            order: PaintUnit::PAINT_ORDER,
            commands: Vec::new(),
        };
        let mut units = vec![a_non_positioned, c_negative];
        units.sort_by_key(|u| (u.z, u.layer, u.tree_index, u.order));
        assert_eq!(units[0].z, -2, "negative-z positioned paints first (below)");
        assert_eq!(units[1].z, 0, "non-positioned paints on top of negative-z");
    }
}
