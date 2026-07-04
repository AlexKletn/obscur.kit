//! Drives layout over the retained tree: [`LayoutCoordinator`].

use std::collections::HashMap;

use obscur_geometry::{BoxConstraints, Insets, Point, Rect, Size};
use obscur_identity::{InternedSymbol, NodeId};
use obscur_layout::algorithm::{
    ChildMeasurer, FlexLayout, FlowLayout, GridLayout, LayoutAlgorithm, LayoutChild,
};
use obscur_layout::model::{
    AlignItems, Dimension, FlexDirection, JustifyContent, LayoutBox, LayoutDisplay, LayoutStyle,
};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{layout_keyword, Length, PropertyId, PropertyValue};
use obscur_tree::tree::RetainedTree;

use super::ComputedStyles;

/// The absolute [`LayoutBox`] of every node the coordinator arranged, keyed by
/// [`NodeId`].
///
/// Unlike the per-node boxes produced *inside* a [`LayoutAlgorithm`] (which are
/// parent-relative), the boxes in this map are in **absolute** viewport
/// coordinates: the coordinator accumulates ancestor offsets as it walks down
/// the tree (spec §4.2).
pub type LayoutBoxes = HashMap<NodeId, LayoutBox>;

/// The layout input [`LayoutStyle`] derived for every node, keyed by [`NodeId`].
///
/// This is the runtime's bridge from `obscur-style`'s [`ComputedStyle`] into
/// `obscur-layout`'s own vocabulary (spec §3.10, §4.2): layout consumes its own
/// [`LayoutStyle`], and the mapping from computed styles is the coordinator's
/// responsibility.
pub type LayoutStyles = HashMap<NodeId, LayoutStyle>;

/// The maximum recursion depth of the layout walk.
///
/// The retained tree is expected to be acyclic, but the coordinator never trusts
/// that: a shared or cyclic child link would otherwise recurse forever. This cap
/// bounds both `measure` and `arrange` so recursion always terminates and never
/// overflows the stack. It is deliberately generous (far deeper than any
/// realistic UI tree) so it never truncates a well-formed tree.
const MAX_DEPTH: u32 = 256;

/// Orchestrates layout over a [`RetainedTree`] (spec §3.10, §4.2).
///
/// The coordinator is **orchestration only**: it owns the tree walk and the
/// [`ComputedStyle`] → [`LayoutStyle`] mapping, but the flow/flex math lives
/// entirely in `obscur-layout`'s [`LayoutAlgorithm`] implementations
/// ([`FlowLayout`], [`FlexLayout`]). Its two responsibilities are:
///
/// 1. **Style mapping** — [`layout_styles_from`](Self::layout_styles_from)
///    derives a per-node [`LayoutStyle`] from each node's [`ComputedStyle`].
/// 2. **Recursive driving** — [`layout`](Self::layout) walks the retained tree
///    and, for each node, invokes the algorithm selected by its `display`,
///    threading a [`ChildMeasurer`] that recursively sizes children, and
///    accumulates absolute [`LayoutBox`]es.
///
/// The walk is **cycle-safe** (bounded by [`MAX_DEPTH`]) and **panic-free**:
/// out-of-range child boxes are handled defensively, and no indexing can panic.
///
/// The coordinator is a stateless zero-sized value; construct it with
/// [`new`](Self::new) or [`Default`].
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutCoordinator;

impl LayoutCoordinator {
    /// Creates a coordinator.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Derives a [`LayoutStyle`] for every node in `computed`.
    ///
    /// Each node's [`ComputedStyle`] is mapped independently via
    /// [`layout_style_from`]; nodes absent from `computed` simply produce no
    /// entry in the result.
    #[must_use]
    pub fn layout_styles_from(&self, computed: &ComputedStyles) -> LayoutStyles {
        computed
            .iter()
            .map(|(&node, style)| (node, layout_style_from(style)))
            .collect()
    }

    /// Lays out the whole tree under `viewport` and returns each node's absolute
    /// [`LayoutBox`].
    ///
    /// Returns an empty map when the tree has no root. Otherwise the root is
    /// arranged at the viewport origin under a tight `viewport` constraint, and
    /// every reachable node's absolute box is written into the result.
    ///
    /// `styles` supplies the per-node [`LayoutStyle`] (typically produced by
    /// [`layout_styles_from`](Self::layout_styles_from)); a node missing from
    /// `styles` is laid out with [`LayoutStyle::default`].
    ///
    /// # Redundant work (Phase 1)
    ///
    /// The selected algorithm runs once per node during `arrange`, and each run
    /// re-measures that node's children (which recurses). This is deterministic
    /// and correct but does redundant measurement; a `LayoutCache` will remove
    /// the redundancy in a later phase (spec §4.2).
    #[must_use]
    pub fn layout(
        &self,
        tree: &RetainedTree,
        styles: &LayoutStyles,
        viewport: Size,
    ) -> LayoutBoxes {
        let mut boxes = LayoutBoxes::new();
        let Some(root) = tree.root() else {
            return boxes;
        };
        arrange(
            tree,
            styles,
            root,
            Point::ORIGIN,
            BoxConstraints::tight(viewport),
            &mut boxes,
            MAX_DEPTH,
        );
        boxes
    }
}

/// Maps a single [`ComputedStyle`] into layout's own [`LayoutStyle`].
///
/// This is the full computed → layout bridge (spec §3.10, §4.2): every field of
/// [`LayoutStyle`] is populated from the corresponding computed property so that
/// flex/grid formatting, the box model, and the flex item/container knobs all
/// engage directly from CSS. A property that is absent (or has the wrong value
/// kind) falls back to the matching [`LayoutStyle::default`] value.
///
/// The mapping, field by field:
///
/// * `display` ← [`PropertyId::Display`] keyword. The computed value is a
///   [`PropertyValue::Keyword`] backed by an [`InternedSymbol`]; it is compared
///   against the reserved [`layout_keyword`] handles — `DISPLAY_FLEX` → `Flex`,
///   `DISPLAY_GRID` → `Grid`, `DISPLAY_NONE` → `None`, everything else (including
///   `DISPLAY_BLOCK` and an absent property) → `Block`. Because both the CSS
///   compiler and this bridge use those reserved symbol values, the identities
///   compare equal across crates without a shared interner.
/// * `width` / `height` / `min_width` / `min_height` / `max_width` /
///   `max_height` ← the matching `PropertyId::{Width, Height, MinWidth,
///   MinHeight, MaxWidth, MaxHeight}` length, via [`dimension_of`]
///   ([`Length::Px`] → [`Dimension::Px`], [`Length::Percent`] →
///   [`Dimension::Percent`], [`Length::Auto`]/absent → [`Dimension::Auto`]).
/// * `margin` / `padding` ← the four `*-top/right/bottom/left` longhands,
///   resolved to pixel [`Insets`] via [`insets_of`]. Insets are pixel-only, so a
///   `Percent`/`Auto` length contributes `0.0` on that edge. `border` stays at
///   its zero default (no CSS border-width property is modelled yet).
/// * `gap` ← [`PropertyId::Gap`] length in pixels (`Percent`/`Auto` → `0.0`).
/// * `flex_grow` / `flex_shrink` ← `PropertyId::{FlexGrow, FlexShrink}` numbers,
///   defaulting to the [`LayoutStyle::default`] values (`0.0` / `1.0`).
/// * `flex_basis` ← [`PropertyId::FlexBasis`] via [`dimension_of`].
/// * `flex_direction` ← [`PropertyId::FlexDirection`] keyword against the
///   reserved `FLEX_*` handles (default `Row`).
/// * `justify_content` ← [`PropertyId::JustifyContent`] keyword against the
///   reserved `JUSTIFY_*` handles (default `FlexStart`).
/// * `align_items` ← [`PropertyId::AlignItems`] keyword against the reserved
///   `ALIGN_*` handles (default `Stretch`).
fn layout_style_from(computed: &ComputedStyle) -> LayoutStyle {
    let default = LayoutStyle::default();
    LayoutStyle {
        display: display_of(computed),
        width: dimension_of(computed, PropertyId::Width),
        height: dimension_of(computed, PropertyId::Height),
        min_width: dimension_of(computed, PropertyId::MinWidth),
        min_height: dimension_of(computed, PropertyId::MinHeight),
        max_width: dimension_of(computed, PropertyId::MaxWidth),
        max_height: dimension_of(computed, PropertyId::MaxHeight),
        margin: insets_of(
            computed,
            PropertyId::MarginTop,
            PropertyId::MarginRight,
            PropertyId::MarginBottom,
            PropertyId::MarginLeft,
        ),
        padding: insets_of(
            computed,
            PropertyId::PaddingTop,
            PropertyId::PaddingRight,
            PropertyId::PaddingBottom,
            PropertyId::PaddingLeft,
        ),
        // No CSS border-width property is modelled yet; keep the zero default.
        border: default.border,
        flex_direction: flex_direction_of(computed),
        justify_content: justify_content_of(computed),
        align_items: align_items_of(computed),
        gap: length_px_of(computed, PropertyId::Gap),
        flex_grow: number_of(computed, PropertyId::FlexGrow, default.flex_grow),
        flex_shrink: number_of(computed, PropertyId::FlexShrink, default.flex_shrink),
        flex_basis: dimension_of(computed, PropertyId::FlexBasis),
    }
}

/// Resolves one length-valued property into a [`Dimension`], defaulting to
/// [`Dimension::Auto`] when the property is absent or not a length.
fn dimension_of(computed: &ComputedStyle, property: PropertyId) -> Dimension {
    match computed.get(property).and_then(ComputedValue::as_length) {
        Some(Length::Px(px)) => Dimension::Px(px.0),
        Some(Length::Percent(percent)) => Dimension::Percent(percent),
        Some(Length::Auto) | None => Dimension::Auto,
    }
}

/// Resolves one length-valued property into a pixel value, defaulting to `0.0`.
///
/// Layout insets and `gap` are pixel-only, so a `Percent` or `Auto` length (or
/// an absent/non-length property) contributes `0.0`.
fn length_px_of(computed: &ComputedStyle, property: PropertyId) -> f32 {
    match computed.get(property).and_then(ComputedValue::as_length) {
        Some(Length::Px(px)) => px.0,
        Some(Length::Percent(_) | Length::Auto) | None => 0.0,
    }
}

/// Builds pixel [`Insets`] from the four edge longhands (top/right/bottom/left).
fn insets_of(
    computed: &ComputedStyle,
    top: PropertyId,
    right: PropertyId,
    bottom: PropertyId,
    left: PropertyId,
) -> Insets {
    Insets {
        top: length_px_of(computed, top),
        right: length_px_of(computed, right),
        bottom: length_px_of(computed, bottom),
        left: length_px_of(computed, left),
    }
}

/// Resolves one number-valued property, defaulting to `default` when the
/// property is absent or not a bare number.
fn number_of(computed: &ComputedStyle, property: PropertyId, default: f32) -> f32 {
    match computed.get(property).map(ComputedValue::value) {
        Some(&PropertyValue::Number(value)) => value,
        _ => default,
    }
}

/// Extracts the interned keyword symbol of a keyword-valued property, or [`None`]
/// when the property is absent or is not a keyword.
fn keyword_sym(computed: &ComputedStyle, property: PropertyId) -> Option<InternedSymbol> {
    match computed.get(property).map(ComputedValue::value) {
        Some(&PropertyValue::Keyword(sym)) => Some(sym),
        _ => None,
    }
}

/// Resolves the computed `display` keyword to a [`LayoutDisplay`].
///
/// Compares the interned symbol against the reserved [`layout_keyword`] handles;
/// an absent or unrecognized value maps to [`LayoutDisplay::Block`] (which also
/// covers `DISPLAY_BLOCK`).
fn display_of(computed: &ComputedStyle) -> LayoutDisplay {
    match keyword_sym(computed, PropertyId::Display) {
        Some(sym) if sym == layout_keyword::DISPLAY_FLEX => LayoutDisplay::Flex,
        Some(sym) if sym == layout_keyword::DISPLAY_GRID => LayoutDisplay::Grid,
        Some(sym) if sym == layout_keyword::DISPLAY_NONE => LayoutDisplay::None,
        _ => LayoutDisplay::Block,
    }
}

/// Resolves the computed `flex-direction` keyword to a [`FlexDirection`],
/// defaulting to [`FlexDirection::Row`].
fn flex_direction_of(computed: &ComputedStyle) -> FlexDirection {
    match keyword_sym(computed, PropertyId::FlexDirection) {
        Some(sym) if sym == layout_keyword::FLEX_ROW_REVERSE => FlexDirection::RowReverse,
        Some(sym) if sym == layout_keyword::FLEX_COLUMN => FlexDirection::Column,
        Some(sym) if sym == layout_keyword::FLEX_COLUMN_REVERSE => FlexDirection::ColumnReverse,
        _ => FlexDirection::Row,
    }
}

/// Resolves the computed `justify-content` keyword to a [`JustifyContent`],
/// defaulting to [`JustifyContent::FlexStart`].
///
/// Note: `space-evenly` has no reserved keyword handle yet, so it cannot be
/// produced here; it falls through to the default.
fn justify_content_of(computed: &ComputedStyle) -> JustifyContent {
    match keyword_sym(computed, PropertyId::JustifyContent) {
        Some(sym) if sym == layout_keyword::JUSTIFY_CENTER => JustifyContent::Center,
        Some(sym) if sym == layout_keyword::JUSTIFY_END => JustifyContent::FlexEnd,
        Some(sym) if sym == layout_keyword::JUSTIFY_SPACE_BETWEEN => JustifyContent::SpaceBetween,
        Some(sym) if sym == layout_keyword::JUSTIFY_SPACE_AROUND => JustifyContent::SpaceAround,
        _ => JustifyContent::FlexStart,
    }
}

/// Resolves the computed `align-items` keyword to an [`AlignItems`], defaulting
/// to [`AlignItems::Stretch`].
fn align_items_of(computed: &ComputedStyle) -> AlignItems {
    match keyword_sym(computed, PropertyId::AlignItems) {
        Some(sym) if sym == layout_keyword::ALIGN_START => AlignItems::FlexStart,
        Some(sym) if sym == layout_keyword::ALIGN_CENTER => AlignItems::Center,
        Some(sym) if sym == layout_keyword::ALIGN_END => AlignItems::FlexEnd,
        _ => AlignItems::Stretch,
    }
}

/// Selects the [`LayoutAlgorithm`] for a `display` mode, or [`None`] when the
/// node generates no box ([`LayoutDisplay::None`]).
///
/// The returned trait object is a zero-sized stateless algorithm; the caller
/// invokes it with the node's style, children, constraints, and a measurer.
fn algorithm_for(display: LayoutDisplay) -> Option<Box<dyn LayoutAlgorithm>> {
    match display {
        LayoutDisplay::Block => Some(Box::new(FlowLayout)),
        LayoutDisplay::Flex => Some(Box::new(FlexLayout)),
        // The layout model carries no grid-template fields yet, so a grid
        // container uses the default single implicit column (spec-faithful when
        // `grid-template-columns` is unspecified). A later phase will thread the
        // real track count once the model represents it.
        LayoutDisplay::Grid => Some(Box::new(GridLayout::new(1))),
        LayoutDisplay::None => None,
    }
}

/// Looks up a node's [`LayoutStyle`], falling back to the default.
fn style_of(styles: &LayoutStyles, node: NodeId) -> LayoutStyle {
    styles.get(&node).copied().unwrap_or_default()
}

/// Collects a node's children as [`LayoutChild`] descriptors (identity + style).
///
/// A node absent from the tree contributes no children. Each child's style is
/// resolved via [`style_of`] so the algorithm sees a complete descriptor.
fn children_of(tree: &RetainedTree, styles: &LayoutStyles, node: NodeId) -> Vec<LayoutChild> {
    tree.get(node).map_or_else(Vec::new, |n| {
        n.children()
            .iter()
            .map(|&child| LayoutChild::new(child, style_of(styles, child)))
            .collect()
    })
}

/// The [`ChildMeasurer`] the coordinator hands to an algorithm.
///
/// It measures a child purely — by recursing into [`measure`] — and never
/// touches the results map, which is what keeps [`arrange`] borrow-clean (the
/// `boxes` map is not borrowed for the duration of the algorithm call).
struct RecMeasurer<'a> {
    /// The retained tree being laid out.
    tree: &'a RetainedTree,
    /// Per-node layout styles.
    styles: &'a LayoutStyles,
    /// The children of the node currently being laid out, parallel to the
    /// indices the algorithm passes to [`ChildMeasurer::measure_child`].
    children: &'a [LayoutChild],
    /// Remaining recursion budget for children measured through this measurer.
    depth: u32,
}

impl ChildMeasurer for RecMeasurer<'_> {
    fn measure_child(&mut self, index: usize, constraints: BoxConstraints) -> Size {
        match self.children.get(index) {
            Some(child) => measure(
                self.tree,
                self.styles,
                child.node(),
                constraints,
                self.depth,
            ),
            // Out-of-range index: contribute nothing rather than panic.
            None => Size::ZERO,
        }
    }
}

/// Pure measure pass: computes `node`'s border-box size under `constraints`
/// without writing anything.
///
/// This mirrors the algorithm selection of [`arrange`] but discards the child
/// boxes and returns only the node size. Because it never touches the results
/// map, it can be called freely from inside an algorithm (via [`RecMeasurer`])
/// while `arrange` holds a mutable borrow of that map.
///
/// `depth` bounds the recursion: at zero it returns [`Size::ZERO`], guaranteeing
/// termination on a cyclic or pathologically deep tree.
fn measure(
    tree: &RetainedTree,
    styles: &LayoutStyles,
    node: NodeId,
    constraints: BoxConstraints,
    depth: u32,
) -> Size {
    if depth == 0 {
        return Size::ZERO;
    }
    let style = style_of(styles, node);
    let Some(algorithm) = algorithm_for(style.display) else {
        return Size::ZERO;
    };
    let children = children_of(tree, styles, node);
    let mut measurer = RecMeasurer {
        tree,
        styles,
        children: &children,
        depth: depth - 1,
    };
    let out = algorithm.layout(&style, &children, constraints, &mut measurer);
    out.size()
}

/// Arrange pass: lays `node` out under `constraints`, writes its **absolute**
/// [`LayoutBox`] (translated by `origin`) into `boxes`, and recurses into each
/// child at its accumulated absolute origin.
///
/// The algorithm is invoked with a [`RecMeasurer`] that only *measures* (pure),
/// so `boxes` is **not** borrowed during the call — after the call returns we
/// are free to mutate `boxes`. This measure/arrange split is what keeps the
/// recursion borrow-clean: measuring never reads or writes the results map, so
/// there is no aliasing between the mutable `boxes` borrow held across the
/// recursion and the shared borrows a measurer needs.
///
/// `depth` bounds the recursion: at zero it returns immediately.
///
/// Child boxes returned by the algorithm are **parent-relative**; each child's
/// absolute origin is `origin + child_box.origin()`. The child count and the
/// returned box count are reconciled defensively so no index can panic.
fn arrange(
    tree: &RetainedTree,
    styles: &LayoutStyles,
    node: NodeId,
    origin: Point,
    constraints: BoxConstraints,
    boxes: &mut LayoutBoxes,
    depth: u32,
) {
    if depth == 0 {
        return;
    }
    let style = style_of(styles, node);
    let Some(algorithm) = algorithm_for(style.display) else {
        return;
    };
    let children = children_of(tree, styles, node);

    // Measurer is pure: it does not borrow `boxes`, so `boxes` is free after.
    let out = {
        let mut measurer = RecMeasurer {
            tree,
            styles,
            children: &children,
            depth: depth - 1,
        };
        algorithm.layout(&style, &children, constraints, &mut measurer)
    };

    // Now safe to write this node's absolute box.
    boxes.insert(node, LayoutBox::new(Rect::new(origin, out.size())));

    // Recurse into children, translating each parent-relative box to absolute.
    // Zipping bounds both sides, so a mismatched length never indexes out of range.
    for (child, child_rel) in children.iter().zip(out.child_boxes().iter()) {
        let rel = child_rel.origin();
        let child_origin = Point::new(origin.x + rel.x, origin.y + rel.y);
        arrange(
            tree,
            styles,
            child.node(),
            child_origin,
            BoxConstraints::tight(child_rel.size()),
            boxes,
            depth - 1,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;
    use obscur_tree::node::{LogicalParent, NodeKind};

    /// Builds a `LayoutStyle` with an explicit block size.
    fn block(width: f32, height: f32) -> LayoutStyle {
        LayoutStyle::default()
            .with_display(LayoutDisplay::Block)
            .with_width(Dimension::Px(width))
            .with_height(Dimension::Px(height))
    }

    /// Allocates an element node under `parent`, wiring the child link and the
    /// logical parent, and returns its id.
    fn add_child(tree: &mut RetainedTree, parent: NodeId, raw_component: u64) -> NodeId {
        let id = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(raw_component),
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
    fn empty_tree_lays_out_to_empty_boxes() {
        let coordinator = LayoutCoordinator::new();
        let tree = RetainedTree::new();
        let styles = LayoutStyles::new();
        let boxes = coordinator.layout(&tree, &styles, Size::new(800.0, 600.0));
        assert!(boxes.is_empty());
    }

    #[test]
    fn mapping_reads_width_and_defaults_display_to_block() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyId::Width, ComputedValue::length(Length::px(120.0)));
        let style = layout_style_from(&computed);
        assert_eq!(style.width, Dimension::Px(120.0));
        // Height absent -> Auto.
        assert_eq!(style.height, Dimension::Auto);
        // Display is always Block in Phase 1 (interner not available).
        assert_eq!(style.display, LayoutDisplay::Block);
    }

    #[test]
    fn mapping_percent_and_auto() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::Width,
            ComputedValue::length(Length::Percent(50.0)),
        );
        computed.set(PropertyId::Height, ComputedValue::length(Length::Auto));
        let style = layout_style_from(&computed);
        assert_eq!(style.width, Dimension::Percent(50.0));
        assert_eq!(style.height, Dimension::Auto);
    }

    #[test]
    fn layout_styles_from_maps_every_node() {
        let coordinator = LayoutCoordinator::new();
        let mut computed = ComputedStyles::new();
        let mut a = ComputedStyle::new();
        a.set(PropertyId::Width, ComputedValue::length(Length::px(10.0)));
        computed.insert(NodeId::from_raw(1), a);
        computed.insert(NodeId::from_raw(2), ComputedStyle::new());

        let styles = coordinator.layout_styles_from(&computed);
        assert_eq!(styles.len(), 2);
        assert_eq!(
            styles.get(&NodeId::from_raw(1)).map(|s| s.width),
            Some(Dimension::Px(10.0))
        );
        assert_eq!(
            styles.get(&NodeId::from_raw(2)).map(|s| s.width),
            Some(Dimension::Auto)
        );
    }

    #[test]
    fn flow_stacking_produces_absolute_boxes() {
        // Block root (width 200) with two fixed-height block children.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child_a = add_child(&mut tree, root, 10);
        let child_b = add_child(&mut tree, root, 11);

        let mut styles = LayoutStyles::new();
        styles.insert(root, block(200.0, 100.0));
        styles.insert(child_a, block(200.0, 50.0));
        styles.insert(child_b, block(200.0, 50.0));

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(&tree, &styles, Size::new(200.0, 100.0));

        // Root at the viewport origin.
        assert_eq!(boxes[&root].origin(), Point::new(0.0, 0.0));
        // First child stacks at y=0, second below it at y=50 (absolute).
        assert_eq!(boxes[&child_a].origin(), Point::new(0.0, 0.0));
        assert_eq!(boxes[&child_a].size(), Size::new(200.0, 50.0));
        assert_eq!(boxes[&child_b].origin(), Point::new(0.0, 50.0));
        assert_eq!(boxes[&child_b].size(), Size::new(200.0, 50.0));
    }

    #[test]
    fn flex_row_places_children_side_by_side() {
        // Flex root (width 200, height 100) with two flex-basis-50 children.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child_a = add_child(&mut tree, root, 20);
        let child_b = add_child(&mut tree, root, 21);

        let flex_item = LayoutStyle::default()
            .with_display(LayoutDisplay::Flex)
            .with_height(Dimension::Px(100.0));
        let flex_item = LayoutStyle {
            flex_basis: Dimension::Px(50.0),
            flex_grow: 0.0,
            ..flex_item
        };

        let mut styles = LayoutStyles::new();
        styles.insert(
            root,
            LayoutStyle {
                display: LayoutDisplay::Flex,
                width: Dimension::Px(200.0),
                height: Dimension::Px(100.0),
                ..LayoutStyle::default()
            },
        );
        styles.insert(child_a, flex_item);
        styles.insert(child_b, flex_item);

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(&tree, &styles, Size::new(200.0, 100.0));

        // Children sit side by side along the main (row) axis: x=0 then x=50.
        assert_eq!(boxes[&child_a].origin(), Point::new(0.0, 0.0));
        assert_eq!(boxes[&child_a].size().width, 50.0);
        assert_eq!(boxes[&child_b].origin().x, 50.0);
        assert_eq!(boxes[&child_b].size().width, 50.0);
    }

    #[test]
    fn nesting_accumulates_absolute_offsets() {
        // root -> container -> leaf; each level indents the child by its own box.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let container = add_child(&mut tree, root, 30);
        let leaf = add_child(&mut tree, container, 31);

        // Give the root padding so the container is offset within it, and the
        // container padding so the leaf is offset within the container. Padding
        // offsets children into the content box in flow layout.
        let mut root_style = block(200.0, 200.0);
        root_style.padding = obscur_geometry::Insets::uniform(10.0);
        let mut container_style = block(100.0, 100.0);
        container_style.padding = obscur_geometry::Insets::uniform(5.0);
        let leaf_style = block(50.0, 50.0);

        let mut styles = LayoutStyles::new();
        styles.insert(root, root_style);
        styles.insert(container, container_style);
        styles.insert(leaf, leaf_style);

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(&tree, &styles, Size::new(200.0, 200.0));

        // Container offset by the root's top/left padding.
        assert_eq!(boxes[&container].origin(), Point::new(10.0, 10.0));
        // Leaf absolute origin = root padding (10) + container padding (5) = 15.
        assert_eq!(boxes[&leaf].origin(), Point::new(15.0, 15.0));
        assert_eq!(boxes[&leaf].size(), Size::new(50.0, 50.0));
    }

    #[test]
    fn mapping_populates_every_field_from_computed_style() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::Display,
            ComputedValue::keyword(layout_keyword::DISPLAY_FLEX),
        );
        // Box model.
        computed.set(PropertyId::MarginTop, ComputedValue::length(Length::px(1.0)));
        computed.set(
            PropertyId::MarginRight,
            ComputedValue::length(Length::px(2.0)),
        );
        computed.set(
            PropertyId::MarginBottom,
            ComputedValue::length(Length::px(3.0)),
        );
        computed.set(
            PropertyId::MarginLeft,
            ComputedValue::length(Length::px(4.0)),
        );
        computed.set(
            PropertyId::PaddingTop,
            ComputedValue::length(Length::px(5.0)),
        );
        computed.set(
            PropertyId::PaddingRight,
            ComputedValue::length(Length::px(6.0)),
        );
        computed.set(
            PropertyId::PaddingBottom,
            ComputedValue::length(Length::px(7.0)),
        );
        computed.set(
            PropertyId::PaddingLeft,
            ComputedValue::length(Length::px(8.0)),
        );
        // Sizing constraints.
        computed.set(PropertyId::MinWidth, ComputedValue::length(Length::px(10.0)));
        computed.set(
            PropertyId::MinHeight,
            ComputedValue::length(Length::px(11.0)),
        );
        computed.set(
            PropertyId::MaxWidth,
            ComputedValue::length(Length::Percent(80.0)),
        );
        computed.set(
            PropertyId::MaxHeight,
            ComputedValue::length(Length::px(13.0)),
        );
        // Flex container + item.
        computed.set(PropertyId::Gap, ComputedValue::length(Length::px(9.0)));
        computed.set(
            PropertyId::FlexDirection,
            ComputedValue::keyword(layout_keyword::FLEX_COLUMN),
        );
        computed.set(
            PropertyId::JustifyContent,
            ComputedValue::keyword(layout_keyword::JUSTIFY_CENTER),
        );
        computed.set(
            PropertyId::AlignItems,
            ComputedValue::keyword(layout_keyword::ALIGN_STRETCH),
        );
        computed.set(PropertyId::FlexGrow, ComputedValue::number(1.0));
        computed.set(PropertyId::FlexShrink, ComputedValue::number(2.0));
        computed.set(
            PropertyId::FlexBasis,
            ComputedValue::length(Length::px(50.0)),
        );

        let style = layout_style_from(&computed);

        assert_eq!(style.display, LayoutDisplay::Flex);
        assert_eq!(
            style.margin,
            Insets {
                top: 1.0,
                right: 2.0,
                bottom: 3.0,
                left: 4.0
            }
        );
        assert_eq!(
            style.padding,
            Insets {
                top: 5.0,
                right: 6.0,
                bottom: 7.0,
                left: 8.0
            }
        );
        assert_eq!(style.min_width, Dimension::Px(10.0));
        assert_eq!(style.min_height, Dimension::Px(11.0));
        assert_eq!(style.max_width, Dimension::Percent(80.0));
        assert_eq!(style.max_height, Dimension::Px(13.0));
        assert_eq!(style.gap, 9.0);
        assert_eq!(style.flex_direction, FlexDirection::Column);
        assert_eq!(style.justify_content, JustifyContent::Center);
        assert_eq!(style.align_items, AlignItems::Stretch);
        assert_eq!(style.flex_grow, 1.0);
        assert_eq!(style.flex_shrink, 2.0);
        assert_eq!(style.flex_basis, Dimension::Px(50.0));
    }

    #[test]
    fn mapping_display_grid_maps_to_grid_and_has_algorithm() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::Display,
            ComputedValue::keyword(layout_keyword::DISPLAY_GRID),
        );
        let style = layout_style_from(&computed);
        assert_eq!(style.display, LayoutDisplay::Grid);
        assert!(algorithm_for(LayoutDisplay::Grid).is_some());
    }

    #[test]
    fn mapping_display_none_maps_to_none() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::Display,
            ComputedValue::keyword(layout_keyword::DISPLAY_NONE),
        );
        assert_eq!(layout_style_from(&computed).display, LayoutDisplay::None);
    }

    #[test]
    fn flex_engages_from_computed_style_end_to_end() {
        // A container with `display: flex; flex-direction: row` and two children
        // with `flex-basis: 50px` must lay the children out side by side once the
        // display keyword is mapped from computed style. Before this bridge every
        // node defaulted to Block and the children would have stacked.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child_a = add_child(&mut tree, root, 40);
        let child_b = add_child(&mut tree, root, 41);

        let mut computed = ComputedStyles::new();

        let mut root_style = ComputedStyle::new();
        root_style.set(
            PropertyId::Display,
            ComputedValue::keyword(layout_keyword::DISPLAY_FLEX),
        );
        root_style.set(
            PropertyId::FlexDirection,
            ComputedValue::keyword(layout_keyword::FLEX_ROW),
        );
        root_style.set(PropertyId::Width, ComputedValue::length(Length::px(200.0)));
        root_style.set(PropertyId::Height, ComputedValue::length(Length::px(100.0)));
        computed.insert(root, root_style);

        for &child in &[child_a, child_b] {
            let mut item = ComputedStyle::new();
            item.set(
                PropertyId::FlexBasis,
                ComputedValue::length(Length::px(50.0)),
            );
            item.set(PropertyId::Height, ComputedValue::length(Length::px(100.0)));
            computed.insert(child, item);
        }

        let coordinator = LayoutCoordinator::new();
        let styles = coordinator.layout_styles_from(&computed);
        // Sanity: the bridge produced a flex container.
        assert_eq!(styles[&root].display, LayoutDisplay::Flex);

        let boxes = coordinator.layout(&tree, &styles, Size::new(200.0, 100.0));

        // Children sit side by side along the row axis: distinct x offsets.
        assert_eq!(boxes[&child_a].origin().x, 0.0);
        assert_eq!(boxes[&child_b].origin().x, 50.0);
        // Same y (a row, not a stack) — proving flex, not block, engaged.
        assert_eq!(boxes[&child_a].origin().y, boxes[&child_b].origin().y);
    }

    #[test]
    fn display_none_generates_no_box_for_subtree() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let mut styles = LayoutStyles::new();
        styles.insert(
            root,
            LayoutStyle::default().with_display(LayoutDisplay::None),
        );

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(&tree, &styles, Size::new(100.0, 100.0));
        // A `None` root produces no box at all.
        assert!(boxes.is_empty());
    }
}
