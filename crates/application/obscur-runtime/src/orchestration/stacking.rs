#![doc = include_str!("../../docs/api/orchestration/stacking.doc/_module.md")]

use std::collections::{HashMap, HashSet};

use obscur_identity::{FastBuildHasher, FastMap, FastSet, NodeId};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{PropertyId, PropertyValue, layout_keyword};
use obscur_tree::tree::RetainedTree;

use super::ComputedStyles;
use super::composite_effects::{blend_mode_of, isolation_of, opacity_of};
use super::context_entry::{ContextEntry, context_entry_key};
use super::effective_stacking::EffectiveStacking;
use super::layout_style_mapping::{keyword_sym, number_of};
use super::paint_layer::PaintLayer;
use super::paint_transform::has_transform;

fn has_filter(style: &ComputedStyle) -> bool {
    [PropertyId::Filter, PropertyId::BackdropFilter]
        .into_iter()
        .any(|property| {
            style
                .get(property)
                .is_some_and(|value| match value.value() {
                    PropertyValue::Filter(ops) => !ops.is_empty(),
                    PropertyValue::FilterFunctions(functions) => !functions.is_empty(),
                    _ => false,
                })
        })
}

fn is_grid_or_flex(style: &ComputedStyle) -> bool {
    matches!(
        keyword_sym(style, PropertyId::Display),
        Some(layout_keyword::DISPLAY_GRID | layout_keyword::DISPLAY_FLEX)
    )
}

/// Visits visual children in reverse paint order for a stack-based walk.
pub(super) fn for_each_paint_child_rev(
    tree: &RetainedTree,
    computed: &ComputedStyles,
    parent: NodeId,
    mut visit: impl FnMut(NodeId),
) {
    let keyword = |node, property| keyword_sym(computed.get(&node)?, property);
    let children = tree.visual_children(parent);
    if !computed.get(&parent).is_some_and(is_grid_or_flex) {
        children.rev().for_each(visit);
        return;
    }

    let order = |node| {
        if matches!(
            keyword(node, PropertyId::Position),
            Some(layout_keyword::POSITION_ABSOLUTE | layout_keyword::POSITION_FIXED)
        ) {
            return 0;
        }
        computed
            .get(&node)
            .map_or(0, |style| number_of(style, PropertyId::Order, 0.0) as i32)
    };
    let is_contents =
        |node| keyword(node, PropertyId::Display) == Some(layout_keyword::DISPLAY_CONTENTS);
    if tree.visual_children(parent).any(is_contents) {
        // The formatting context promotes children of boxless wrappers to
        // siblings before applying order. Keep that same sequence for painting.
        let mut pending: Vec<_> = children.rev().collect();
        let mut unwrapped = FastSet::default();
        let mut items = Vec::new();
        while let Some(child) = pending.pop() {
            if is_contents(child) {
                if unwrapped.insert(child) {
                    pending.extend(tree.visual_children(child).rev());
                }
            } else {
                items.push((order(child), items.len(), child));
            }
        }
        items.sort_unstable_by_key(|&(order, source_index, _)| (order, source_index));
        for (_, _, child) in items.into_iter().rev() {
            visit(child);
        }
        return;
    }

    let mut previous = i32::MIN;
    let already_ordered = tree.visual_children(parent).all(|child| {
        let next = order(child);
        let ordered = previous <= next;
        previous = next;
        ordered
    });
    if already_ordered {
        children.rev().for_each(visit);
        return;
    }
    let mut reordered: Vec<_> = children
        .enumerate()
        .map(|(source_index, child)| (order(child), source_index, child))
        .collect();
    reordered.sort_unstable_by_key(|&(order, index, _)| (order, index));
    for (_, _, child) in reordered.into_iter().rev() {
        visit(child);
    }
}

/// Gives each local stacking group a paint band in the flattened context tree.
fn assign_paint_bands(
    root: NodeId,
    mut contexts: FastMap<NodeId, Vec<ContextEntry>>,
    out: &mut EffectiveStacking,
) {
    enum Step {
        Enter(NodeId),
        Paint(NodeId, ContextEntry),
        Exit(NodeId),
    }
    let mut pending = vec![Step::Enter(root)];
    let mut band = -1i32;
    let mut previous = None;
    while let Some(step) = pending.pop() {
        match step {
            Step::Enter(node) => {
                band = band.saturating_add(1);
                out.paint_band.insert(node, band);
                previous = None;
                pending.push(Step::Exit(node));
                let mut entries = contexts.remove(&node).unwrap_or_default();
                entries.sort_unstable_by_key(context_entry_key);
                pending.extend(
                    entries
                        .into_iter()
                        .rev()
                        .map(|entry| Step::Paint(node, entry)),
                );
            }
            Step::Paint(_, entry) if entry.context => pending.push(Step::Enter(entry.node)),
            Step::Paint(parent, entry) => {
                let key = (parent, entry.z, entry.layer);
                if previous != Some(key) {
                    band = band.saturating_add(1);
                    previous = Some(key);
                }
                out.paint_band.insert(entry.node, band);
            }
            Step::Exit(node) => {
                out.context_end_band.insert(node, band);
                previous = None;
            }
        }
    }
}

#[doc = include_str!("../../docs/api/orchestration/stacking.doc/effective_stacking.md")]
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
    // One entry per live node goes into each of the three tables, so their final
    // size is known before the walk starts. Reserving it costs one sized
    // allocation apiece and saves each table the ~7 rehashes it otherwise pays
    // growing from empty to a couple of hundred entries.
    let capacity = tree.len();
    let mut out = EffectiveStacking {
        z: HashMap::with_capacity_and_hasher(capacity, FastBuildHasher::default()),
        #[cfg(test)]
        layer: HashMap::with_capacity_and_hasher(capacity, FastBuildHasher::default()),
        paint_index: HashMap::with_capacity_and_hasher(capacity, FastBuildHasher::default()),
        paint_band: HashMap::with_capacity_and_hasher(capacity, FastBuildHasher::default()),
        context_end_band: HashMap::default(),
    };
    let Some(root) = tree.root() else {
        return out;
    };
    let mut visited: FastSet<NodeId> =
        HashSet::with_capacity_and_hasher(capacity, FastBuildHasher::default());
    let mut contexts: FastMap<NodeId, Vec<ContextEntry>> = HashMap::default();
    let mut stack = vec![(
        root,
        0,
        PaintLayer::NonPositioned,
        false,
        root,
        PaintLayer::NonPositioned,
    )];
    while let Some((node, inherited_z, inherited_layer, layout_item, context, local_layer)) =
        stack.pop()
    {
        if !visited.insert(node) {
            continue;
        }
        let style = computed.get(&node);
        let own = style.and_then(|style| own_stacking(style, layout_item));
        let z = own.and_then(|(z, _)| z).unwrap_or(inherited_z);
        let layer = own.map_or(inherited_layer, |(_, layer)| layer);
        out.z.insert(node, z);
        #[cfg(test)]
        out.layer.insert(node, layer);
        let index = visited.len() - 1;
        out.paint_index.insert(node, index);
        let own_z = own.and_then(|(z, _)| z);
        let effect_context = isolation_of(style)
            || opacity_of(style).is_some()
            || blend_mode_of(style).is_some()
            || style.is_some_and(|style| {
                has_transform(style)
                    || has_filter(style)
                    || style.get(PropertyId::ClipPath).is_some_and(|value| {
                        matches!(
                            value.value(),
                            PropertyValue::ClipPolygon { .. }
                                | PropertyValue::ClipEllipse { .. }
                                | PropertyValue::ClipRectangle { .. }
                                | PropertyValue::ClipSvgPath { .. }
                                | PropertyValue::ClipBox(_)
                        )
                    })
            });
        let starts_context = own_z.is_some() || effect_context;
        let local_layer = if own_z.is_none() && effect_context {
            PaintLayer::PositionedZeroOrAuto
        } else {
            own.map_or(local_layer, |(_, layer)| layer)
        };
        if node != root {
            contexts.entry(context).or_default().push(ContextEntry {
                node,
                z: own_z.unwrap_or(0),
                layer: local_layer,
                index,
                context: starts_context,
            });
        }
        let (child_context, child_layer) = if node == root || starts_context {
            (node, PaintLayer::NonPositioned)
        } else {
            (context, local_layer)
        };
        let children_are_layout_items = style.is_some_and(is_grid_or_flex);
        for_each_paint_child_rev(tree, computed, node, |child| {
            if !visited.contains(&child) {
                stack.push((
                    child,
                    z,
                    layer,
                    children_are_layout_items,
                    child_context,
                    child_layer,
                ));
            }
        });
    }
    assign_paint_bands(root, contexts, &mut out);
    out
}

/// Returns the node's own z-index and paint layer. Numeric z-index applies
/// to positioned boxes and to Grid or Flexbox items. Fixed and sticky boxes
/// with auto z-index establish a zero-level context. Relative and absolute
/// boxes with auto z-index share their ancestor's context.
fn own_stacking(style: &ComputedStyle, layout_item: bool) -> Option<(Option<i32>, PaintLayer)> {
    let position = keyword_sym(style, PropertyId::Position);
    let fixed_or_sticky = matches!(
        position,
        Some(layout_keyword::POSITION_FIXED | layout_keyword::POSITION_STICKY)
    );
    let positioned = fixed_or_sticky
        || matches!(
            position,
            Some(layout_keyword::POSITION_RELATIVE | layout_keyword::POSITION_ABSOLUTE)
        );
    match style.get(PropertyId::ZIndex).map(ComputedValue::value) {
        Some(&PropertyValue::Number(n)) if positioned || layout_item => {
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
        _ if fixed_or_sticky => Some((Some(0), PaintLayer::PositionedZeroOrAuto)),
        // Relative and absolute boxes with auto z-index share their ancestor's level.
        _ if positioned => Some((None, PaintLayer::PositionedZeroOrAuto)),
        _ => None,
    }
}

#[doc = include_str!("../../docs/api/orchestration/stacking.doc/z_index_to_i32.md")]
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
        let a_non_positioned = ContextEntry {
            node: NodeId::from_raw(1),
            context: false,
            z: 0,
            layer: PaintLayer::NonPositioned,
            index: 2, // later in the tree
        };
        let b_positioned = ContextEntry {
            node: NodeId::from_raw(1),
            context: false,
            z: 0,
            layer: PaintLayer::PositionedZeroOrAuto,
            index: 1, // earlier in the tree
        };
        let mut units = vec![b_positioned, a_non_positioned];
        units.sort_by_key(context_entry_key);
        // A (non-positioned, later) paints first; B (positioned, earlier) paints
        // last — on top.
        assert_eq!(units[0].index, 2, "non-positioned A paints first");
        assert_eq!(units[1].index, 1, "positioned B paints last (on top)");
    }

    #[test]
    fn sort_key_keeps_non_positioned_in_tree_order() {
        // Regression guard at the sort level: equal (z, layer) units keep
        // `tree_index` (document) order — byte-identical to the pre-fix sequence.
        let mut units = vec![
            ContextEntry {
                node: NodeId::from_raw(1),
                context: false,
                z: 0,
                layer: PaintLayer::NonPositioned,
                index: 3,
            },
            ContextEntry {
                node: NodeId::from_raw(1),
                context: false,
                z: 0,
                layer: PaintLayer::NonPositioned,
                index: 1,
            },
            ContextEntry {
                node: NodeId::from_raw(1),
                context: false,
                z: 0,
                layer: PaintLayer::NonPositioned,
                index: 2,
            },
        ];
        units.sort_by_key(context_entry_key);
        assert_eq!(
            units.iter().map(|u| u.index).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn sort_key_negative_z_paints_below_non_positioned() {
        // A negative-z positioned box (C) sorts below a non-positioned block (A) via
        // the PRIMARY `z` key — the layer rank is not even consulted across z.
        let a_non_positioned = ContextEntry {
            node: NodeId::from_raw(1),
            context: false,
            z: 0,
            layer: PaintLayer::NonPositioned,
            index: 1,
        };
        let c_negative = ContextEntry {
            node: NodeId::from_raw(1),
            context: false,
            z: -2,
            layer: PaintLayer::NonPositioned,
            index: 2,
        };
        let mut units = vec![a_non_positioned, c_negative];
        units.sort_by_key(context_entry_key);
        assert_eq!(units[0].z, -2, "negative-z positioned paints first (below)");
        assert_eq!(units[1].z, 0, "non-positioned paints on top of negative-z");
    }
}
