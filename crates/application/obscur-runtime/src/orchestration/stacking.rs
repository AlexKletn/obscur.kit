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
//! On top of the effective z, each node also carries a **§9.9 paint-layer rank**
//! (see [`PaintLayer`] / [`effective_paint_layer`]) that orders units *within an
//! equal-z bucket*. CSS 2.1 §9.9.1 / Appendix E paint a stacking context's
//! contents in seven layers, bottom→top: (1) the context root's background &
//! borders; (2) child contexts with negative z; (3) non-positioned in-flow
//! **block** descendants; (4) non-positioned **floats**; (5) non-positioned
//! **inline** descendants; (6) **positioned** descendants with `z-index: auto`
//! **or** `0`; (7) child contexts with positive z. The critical consequence — and
//! the one the old flat `(z, tree_index)` sort got wrong (catalog bug B2) — is
//! that **layer 6 always paints above layers 1–5 regardless of tree order**: a
//! positioned `z:auto`/`0` box lifts above its *non-positioned* siblings even when
//! it appears earlier in the document. Negative- and positive-z positioned boxes
//! are already separated by the primary `z` key (layers 2 and 7), so the rank only
//! has to lift positioned-`z0`/`auto` boxes (layer 6) above non-positioned content
//! (layers 1–5) in the common `z == 0` bucket.
//!
//! **Not modelled** (documented simplifications): nested stacking-context
//! isolation (a child's high z can escape its parent's context, unlike real
//! CSS) — the [`effective_paint_layer`] walk *inherits* the lift so a positioned
//! subtree moves together, but it does not re-isolate a descending context; the
//! `auto`-establishes-no-context nuance; opacity/transform/filter induced stacking
//! contexts; and — because `float` is not a computed property here and the
//! byte-identical regression guard forbids reordering non-positioned block vs
//! inline siblings — the §9.9 non-positioned layers 1/3/4/5 are **collapsed into a
//! single rank** ([`PaintLayer::NonPositioned`]). When no node authors a
//! non-`auto` `z-index` *and* no node is positioned, every effective z is `0` and
//! every layer rank is [`PaintLayer::NonPositioned`], so the stable sort by
//! `tree_index` reproduces the exact pre-order paint sequence — the regression
//! guard.

use std::collections::{HashMap, HashSet};

use obscur_identity::NodeId;
use obscur_scene::command::PaintCommand;
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{PropertyId, PropertyValue, layout_keyword};
use obscur_tree::tree::RetainedTree;

use super::ComputedStyles;

/// One node's own paint commands (or a clip bracket), tagged with its stacking
/// key.
///
/// The commands are exactly the node's own background / border / text (never its
/// descendants'), captured in emission order — or, for a `border-radius` element,
/// a lone `PushClip` / `PopClip` that brackets the node's subtree. The key
/// `(z, layer, tree_index, order)` orders the unit against every other unit: `z`
/// is the node's effective stacking level, `layer` its §9.9 paint-layer rank
/// (lifting positioned-`z0`/`auto` boxes above non-positioned content within an
/// equal z bucket), `tree_index` its pre-order document position (the tie-breaker
/// within a z+layer bucket), and `order` a within-position phase so a subtree's
/// `PushClip` sorts before the node's paint and its `PopClip` sorts after the
/// whole subtree (see [`PaintUnit::PAINT_ORDER`]). When nothing is positioned
/// every `layer` is [`PaintLayer::NonPositioned`] and, with no radius, every unit
/// is [`PAINT_ORDER`](PaintUnit::PAINT_ORDER), so the key reduces to
/// `(z, tree_index)` — the pre-fix ordering, byte-for-byte.
pub(super) struct PaintUnit {
    /// The node's effective stacking z (see [`effective_stacking_z`]).
    pub z: i32,
    /// The node's CSS 2.1 §9.9 paint-layer rank (see [`effective_paint_layer`]),
    /// sorted *between* `z` and `tree_index`: within one z bucket a
    /// [`PaintLayer::PositionedZeroOrAuto`] box (layer 6) sorts after every
    /// [`PaintLayer::NonPositioned`] unit (layers 1–5), so a positioned `z:auto`/`0`
    /// box paints above its non-positioned siblings regardless of tree order. Every
    /// bracket/paint unit of one node shares that node's `layer`, so a node's own
    /// clip/transform/opacity/paint interleave is untouched (they tie on `z` and
    /// `layer` and stay ordered by `order`).
    pub layer: PaintLayer,
    /// The node's pre-order document position (for a `PopClip` bracket, the
    /// subtree's *exit* index), tie-breaking equal-z units so ties keep tree order.
    pub tree_index: usize,
    /// The within-position phase (see [`PaintUnit::PAINT_ORDER`]).
    pub order: i64,
    /// The node's own paint commands (or the clip bracket), in emission order.
    pub commands: Vec<PaintCommand>,
}

impl PaintUnit {
    /// A subtree's opening `PushLayer` for a CSS `filter: blur(r)` group layer,
    /// sorted before **every** other bracket at the same document position so the
    /// blur is the OUTERMOST subtree wrapper (outside the transform, opacity, and
    /// clip). The Vello renderer's owned gaussian pass then operates over the
    /// fully-composited element. Below
    /// [`TRANSFORM_PUSH_ORDER`](Self::TRANSFORM_PUSH_ORDER).
    pub(super) const BLUR_PUSH_ORDER: i64 = -4;
    /// A subtree's opening `PushTransform`, sorted before both the `PushLayer`
    /// and the `PushClip` at the same document position so a CSS 2D `transform`
    /// brackets the whole subtree *outside* any opacity group layer or
    /// rounded-corner clip (the transform is the outermost wrapper, save for a
    /// blur layer). Between [`BLUR_PUSH_ORDER`](Self::BLUR_PUSH_ORDER) and
    /// [`LAYER_PUSH_ORDER`](Self::LAYER_PUSH_ORDER).
    pub(super) const TRANSFORM_PUSH_ORDER: i64 = -3;
    /// A subtree's opening `PushLayer` (a CSS `opacity` group layer), sorted
    /// *after* the `PushTransform` but *before* the `PushClip` at the same
    /// document position, so the opacity layer nests inside the transform and
    /// outside the rounded clip (transform → layer → clip). Between
    /// [`TRANSFORM_PUSH_ORDER`](Self::TRANSFORM_PUSH_ORDER) and
    /// [`CLIP_PUSH_ORDER`](Self::CLIP_PUSH_ORDER).
    pub(super) const LAYER_PUSH_ORDER: i64 = -2;
    /// A subtree's opening `PushClip`, sorted before the node's own paint at the
    /// same document position. Pre-order enter indices are unique per node, so
    /// pushes never collide; any value below [`PAINT_ORDER`](Self::PAINT_ORDER)
    /// works.
    pub(super) const CLIP_PUSH_ORDER: i64 = -1;
    /// A node's own paint (background / border / text). The default phase.
    pub(super) const PAINT_ORDER: i64 = 0;

    /// A scroll container's opening `PushTransform(translate(0, −scroll_top))`,
    /// sorted *after* the node's own paint ([`PAINT_ORDER`](Self::PAINT_ORDER)) at
    /// the same document position so the container's own background/border stays
    /// **fixed** in the scrollport while its descendants (all at a strictly greater
    /// `tree_index`) are translated by the scroll offset. Inside the overflow clip
    /// (whose push is at [`CLIP_PUSH_ORDER`](Self::CLIP_PUSH_ORDER)) so scrolled
    /// content is clipped to the scrollport (CSS Overflow 3 §scrollport). Any value
    /// above `PAINT_ORDER` works — enter indices are unique per node.
    pub(super) const SCROLL_TRANSFORM_PUSH_ORDER: i64 = 1;

    /// A scroll container's vertical **scrollbar** (track + thumb) at its subtree
    /// *exit* index. Painted on top of, and outside, the container's whole scrolled
    /// subtree — so it does not scroll — using the same ceiling as
    /// [`OUTLINE_ORDER`](Self::OUTLINE_ORDER); emitted after every other bracket so
    /// a tie at a depth-0 pop still resolves in the scrollbar's favour.
    pub(super) const SCROLLBAR_ORDER: i64 = i64::MAX;

    /// A node's CSS `outline` (focus ring) at its subtree *exit* index. The
    /// outline paints OUTSIDE the node's rounded clip and ON TOP of the node's
    /// whole subtree, so it sorts after every `Pop*` bracket at that index. Set to
    /// [`i64::MAX`] — the ceiling — so it is `>=` every `*_pop_order` (which are
    /// `i64::MAX - depth`); a tie with a depth-0 pop is broken by emitting the
    /// outline bracket LAST (after all pops) so the stable sort keeps it after them.
    pub(super) const OUTLINE_ORDER: i64 = i64::MAX;

    /// The phase for a subtree's closing `PopClip` at its *exit* index: sorts
    /// after the node's paint and after every descendant (all at
    /// [`PAINT_ORDER`](Self::PAINT_ORDER)), and — when several clips end at the
    /// same index (a rightmost-leaf chain) — deeper clips close first
    /// (descending `depth`), so nested clips unwind in the correct order.
    #[must_use]
    pub(super) const fn clip_pop_order(depth: u32) -> i64 {
        i64::MAX - depth as i64
    }

    /// The phase for a subtree's closing `PopLayer` (a CSS `opacity` group
    /// layer) at its *exit* index. Uses the same depth-descending schedule as
    /// [`clip_pop_order`](Self::clip_pop_order) so nested layers unwind
    /// deepest-first; a layer, a clip, and a transform that end at the same index
    /// and depth tie, broken by emitting the brackets in `clip, layer, transform`
    /// order so the clip closes first (innermost), then the layer, then the
    /// transform (outermost).
    #[must_use]
    pub(super) const fn layer_pop_order(depth: u32) -> i64 {
        i64::MAX - depth as i64
    }

    /// The phase for a subtree's closing `PopTransform` at its *exit* index. Uses
    /// the same depth-descending schedule as [`clip_pop_order`](Self::clip_pop_order)
    /// so nested transforms unwind deepest-first; a transform and a clip that end
    /// at the same index and depth tie, broken by emitting the transform bracket
    /// **after** the clip bracket (so the clip closes first, inside the transform).
    #[must_use]
    pub(super) const fn transform_pop_order(depth: u32) -> i64 {
        i64::MAX - depth as i64
    }

    /// The phase for a subtree's closing `PopLayer` (a CSS `filter: blur` group
    /// layer) at its *exit* index. Uses the same depth-descending schedule as
    /// [`clip_pop_order`](Self::clip_pop_order); a blur, transform, opacity, and
    /// clip that end at the same index and depth tie, broken by emitting the blur
    /// bracket **last** so it closes after them all (the blur is the outermost
    /// wrapper).
    #[must_use]
    pub(super) const fn blur_pop_order(depth: u32) -> i64 {
        i64::MAX - depth as i64
    }
}

/// A node's CSS 2.1 §9.9 / Appendix E **paint-layer rank** — the order key used
/// *within a single z bucket* to lift positioned boxes above non-positioned
/// content.
///
/// §9.9.1 paints a stacking context bottom→top in seven layers; layers 2 (negative
/// z) and 7 (positive z) are already separated here by the primary effective-`z`
/// key, so this rank only distinguishes the `z == 0` layers:
///
/// * [`NonPositioned`](Self::NonPositioned) — the context root's background &
///   borders (layer 1) and every non-positioned in-flow **block** (3), **float**
///   (4) and **inline** (5) descendant. These four §9.9 layers are **collapsed
///   into one rank** because `float` is not a computed property in this engine and
///   because splitting block (3) from inline (5) would reorder non-positioned
///   siblings out of document order — violating the byte-identical regression
///   guard. Within this rank `tree_index` (document order) decides.
/// * [`PositionedZeroOrAuto`](Self::PositionedZeroOrAuto) — a **positioned**
///   descendant (and, by inheritance, its subtree) whose effective own `z-index`
///   is `auto` or `0` (§9.9 layer 6). It sorts **after** every `NonPositioned`
///   unit in the same z bucket, so the whole positioned box paints on top of its
///   non-positioned siblings even when it comes earlier in the tree — the fix for
///   catalog bug B2.
///
/// A positioned box with a non-zero integer `z-index` is *not* lifted here: the
/// primary `z` key already floats it into its own bucket (layer 2 or 7), and
/// keeping it at `NonPositioned` leaves it below its own descendants (which share
/// its z) in document order — matching "the SC root's own background paints first"
/// (§9.9 layer 1 within its own context).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum PaintLayer {
    /// §9.9 layers 1/3/4/5 — the context root's bg+borders and all non-positioned
    /// in-flow descendants (collapsed; see the type docs). Sorts *below*
    /// [`PositionedZeroOrAuto`](Self::PositionedZeroOrAuto).
    NonPositioned,
    /// §9.9 layer 6 — a positioned descendant with effective own `z-index` `auto`
    /// or `0` (and its subtree). Sorts *above* [`NonPositioned`](Self::NonPositioned)
    /// within the same z bucket.
    PositionedZeroOrAuto,
}

/// Computes each node's **effective §9.9 paint-layer rank** for the whole tree.
///
/// Walks pre-order from the root (cycle-safe via a visited set); the root's rank
/// is [`PaintLayer::NonPositioned`]. Each node's rank is its **own** rank when it
/// is positioned (see [`own_paint_layer`]), otherwise it **inherits** its parent's
/// rank. Inheritance makes a positioned `z:auto`/`0` box's *whole subtree* share
/// the [`PositionedZeroOrAuto`](PaintLayer::PositionedZeroOrAuto) lift, so the
/// entire positioned box (background, borders and non-positioned descendants) rises
/// above non-positioned siblings **together**, while `tree_index` keeps the
/// ancestor painting below its own descendants. (Nested-context re-isolation — a
/// deeper positioned integer-z context resetting the rank — is applied when that
/// child is itself positioned, but a descendant's high z otherwise escaping its
/// parent context stays the documented simplification.)
///
/// When no node is positioned every rank is
/// [`NonPositioned`](PaintLayer::NonPositioned), so the layer key is uniform and
/// the sort reduces to `tree_index` order — the byte-identical regression guard.
#[must_use]
pub(super) fn effective_paint_layer(
    tree: &RetainedTree,
    computed: &ComputedStyles,
) -> HashMap<NodeId, PaintLayer> {
    let mut map: HashMap<NodeId, PaintLayer> = HashMap::new();
    let Some(root) = tree.root() else {
        return map;
    };
    let mut visited: HashSet<NodeId> = HashSet::new();
    // Each stack entry carries the layer rank inherited from its parent chain.
    let mut stack: Vec<(NodeId, PaintLayer)> = vec![(root, PaintLayer::NonPositioned)];
    while let Some((node, inherited)) = stack.pop() {
        if !visited.insert(node) {
            continue;
        }
        let layer = own_paint_layer(computed, node).unwrap_or(inherited);
        map.insert(node, layer);
        if let Some(retained) = tree.get(node) {
            for &child in retained.children().iter().rev() {
                if !visited.contains(&child) {
                    stack.push((child, layer));
                }
            }
        }
    }
    map
}

/// Reads a node's **own** §9.9 paint-layer rank, or [`None`] when the node does not
/// establish one and should inherit its parent's rank.
///
/// A non-positioned node returns [`None`] (it participates in its parent's layer).
/// A **positioned** node returns [`PaintLayer::PositionedZeroOrAuto`] when its
/// effective own `z-index` is `auto` (or absent) or the integer `0` — the §9.9
/// layer-6 lift; a positioned node with a non-zero integer `z-index` returns
/// [`PaintLayer::NonPositioned`] (it is isolated by the primary `z` key and must
/// not lift above its own descendants — see [`PaintLayer`]).
fn own_paint_layer(computed: &ComputedStyles, node: NodeId) -> Option<PaintLayer> {
    let style = computed.get(&node)?;
    if !is_positioned(style) {
        return None;
    }
    match style.get(PropertyId::ZIndex).map(ComputedValue::value) {
        // A non-zero integer z-index is isolated by the primary `z` key.
        Some(&PropertyValue::Number(n)) if z_index_to_i32(n) != 0 => {
            Some(PaintLayer::NonPositioned)
        }
        // `auto` (absent / non-numeric) or an explicit `0`: §9.9 layer 6.
        _ => Some(PaintLayer::PositionedZeroOrAuto),
    }
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
        let mut computed = ComputedStyles::new();
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
        let mut computed = ComputedStyles::new();
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
        let mut computed = ComputedStyles::new();
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
        let mut computed = ComputedStyles::new();
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
