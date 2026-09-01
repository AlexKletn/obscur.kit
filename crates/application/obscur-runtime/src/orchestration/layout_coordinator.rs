#![doc = include_str!("layout_coordinator.doc/_module.md")]

use obscur_geometry::{BoxConstraints, Point, Rect, Size};
use obscur_identity::{FastMap, NodeId};
use obscur_layout::invalidation::LayoutInvalidation;
use obscur_layout::model::{Dimension, LayoutBox, LayoutStyle};
use obscur_text::layout::Hyphenator;
use obscur_text::shaping::ParagraphShaper;
use obscur_tree::tree::RetainedTree;

use super::arrange_cache::ArrangeCache;
use super::arrange_pass::arrange;
use super::layout_eviction::invalidated_layout;
use super::layout_style_mapping::layout_style_from;
use super::measure_cache::MeasureCache;
use super::{
    ComputedStyles, EditingView, GridTemplates, grid_templates_from, resolve_named_grid_areas,
    resolve_named_grid_lines,
};

#[doc = include_str!("layout_coordinator.doc/LayoutBoxes.md")]
pub type LayoutBoxes = FastMap<NodeId, LayoutBox>;

#[doc = include_str!("layout_coordinator.doc/LayoutStyles.md")]
pub type LayoutStyles = FastMap<NodeId, LayoutStyle>;

#[doc = include_str!("layout_coordinator.doc/MAX_DEPTH.md")]
pub(super) const MAX_DEPTH: u32 = 256;

#[doc = include_str!("layout_coordinator.doc/LayoutCoordinator.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutCoordinator;

impl LayoutCoordinator {
    #[doc = include_str!("layout_coordinator.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("layout_coordinator.doc/layout_styles_from.md")]
    #[must_use]
    pub fn layout_styles_from(
        &self,
        computed: &ComputedStyles,
        viewport: Size,
        root_font_size: f32,
        paragraph: Option<&ParagraphShaper>,
    ) -> LayoutStyles {
        computed
            .iter()
            .map(|(&node, style)| {
                (
                    node,
                    layout_style_from(style, viewport, root_font_size, paragraph),
                )
            })
            .collect()
    }

    #[doc = include_str!("layout_coordinator.doc/layout.md")]
    // The tree, the per-node side-tables, the viewport + scroll offset, and the
    // shaper are the cohesive inputs of one layout entry point.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn layout(
        &self,
        tree: &RetainedTree,
        styles: &LayoutStyles,
        computed: &ComputedStyles,
        grid_templates: &GridTemplates,
        viewport: Size,
        scroll_offset: (f32, f32),
        paragraph: &ParagraphShaper,
    ) -> LayoutBoxes {
        // The default path never hyphenates: [`None`] makes text-leaf measurement
        // byte-identical to whitespace-only wrapping (no hyphen glyph is shaped).
        self.layout_with_hyphenator(
            tree,
            styles,
            computed,
            grid_templates,
            viewport,
            scroll_offset,
            paragraph,
            None,
        )
    }

    #[doc = include_str!("layout_coordinator.doc/grid_templates_from.md")]
    #[must_use]
    pub fn grid_templates_from(
        &self,
        computed: &ComputedStyles,
        viewport: Size,
        root_font_size: f32,
        paragraph: Option<&ParagraphShaper>,
    ) -> GridTemplates {
        grid_templates_from(computed, viewport, root_font_size, paragraph)
    }

    #[doc = include_str!("layout_coordinator.doc/resolve_named_grid_areas.md")]
    pub fn resolve_named_grid_areas(
        &self,
        styles: &mut LayoutStyles,
        tree: &RetainedTree,
        computed: &ComputedStyles,
    ) {
        resolve_named_grid_areas(styles, tree, computed);
    }

    #[doc = include_str!("layout_coordinator.doc/resolve_named_grid_lines.md")]
    pub fn resolve_named_grid_lines(
        &self,
        styles: &mut LayoutStyles,
        tree: &RetainedTree,
        computed: &ComputedStyles,
    ) {
        resolve_named_grid_lines(styles, tree, computed);
    }

    #[doc = include_str!("layout_coordinator.doc/layout_with_hyphenator.md")]
    // The tree, the three per-node side-tables (styles, computed, grid templates),
    // the viewport + scroll offset, and the shaping inputs (paragraph + hyphenator)
    // are cohesive inputs of one layout entry point, not an incidental over-long
    // parameter list.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn layout_with_hyphenator(
        &self,
        tree: &RetainedTree,
        styles: &LayoutStyles,
        computed: &ComputedStyles,
        grid_templates: &GridTemplates,
        viewport: Size,
        scroll_offset: (f32, f32),
        paragraph: &ParagraphShaper,
        hyphenator: Option<&dyn Hyphenator>,
    ) -> LayoutBoxes {
        // No editing session: every text leaf measures from the retained tree,
        // byte-identical to before the editing-aware measure path existed.
        self.layout_editing(
            tree,
            styles,
            computed,
            grid_templates,
            viewport,
            scroll_offset,
            paragraph,
            hyphenator,
            None,
        )
    }

    #[doc = include_str!("layout_coordinator.doc/layout_editing.md")]
    // The editing view is the one added optional input over `layout_with_hyphenator`;
    // the rest are the same cohesive layout inputs.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn layout_editing(
        &self,
        tree: &RetainedTree,
        styles: &LayoutStyles,
        computed: &ComputedStyles,
        grid_templates: &GridTemplates,
        viewport: Size,
        scroll_offset: (f32, f32),
        paragraph: &ParagraphShaper,
        hyphenator: Option<&dyn Hyphenator>,
        editing: Option<EditingView<'_>>,
    ) -> LayoutBoxes {
        // One measure memoization and one arrange cache for this pass only (see
        // [`MeasureCache`] / [`ArrangeCache`]): both start empty, so every node is
        // measured and arranged from scratch, exactly as before either cache could
        // outlive a frame.
        self.arrange_all(
            tree,
            styles,
            computed,
            grid_templates,
            viewport,
            scroll_offset,
            paragraph,
            hyphenator,
            editing,
            &MeasureCache::new(),
            &ArrangeCache::new(),
        )
    }

    #[doc = include_str!("layout_coordinator.doc/layout_invalidated.md")]
    // The same cohesive layout inputs as `layout_editing`, plus the two this entry
    // point exists for.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn layout_invalidated(
        &self,
        tree: &RetainedTree,
        styles: &LayoutStyles,
        computed: &ComputedStyles,
        grid_templates: &GridTemplates,
        viewport: Size,
        scroll_offset: (f32, f32),
        paragraph: &ParagraphShaper,
        hyphenator: Option<&dyn Hyphenator>,
        editing: Option<EditingView<'_>>,
        cache: &MeasureCache,
        arrange: &ArrangeCache,
        dirty: Option<&LayoutInvalidation>,
    ) -> LayoutBoxes {
        // Act on the dirty set: drop exactly the entries it can have invalidated,
        // and drop everything when it cannot bound them (or when there is no set).
        // One classification feeds both caches, which is what keeps them consistent
        // with each other rather than by coincidence.
        match dirty.and_then(|dirty| invalidated_layout(tree, dirty)) {
            Some(evicted) => {
                cache.evict(&evicted.measures);
                arrange.evict(&evicted.arranges);
            }
            None => {
                cache.clear();
                arrange.clear();
            }
        }
        self.arrange_all(
            tree,
            styles,
            computed,
            grid_templates,
            viewport,
            scroll_offset,
            paragraph,
            hyphenator,
            editing,
            cache,
            arrange,
        )
    }

    #[doc = include_str!("layout_coordinator.doc/arrange_all.md")]
    // The cohesive layout inputs of one walk, plus the cache it measures through.
    #[allow(clippy::too_many_arguments)]
    fn arrange_all(
        &self,
        tree: &RetainedTree,
        styles: &LayoutStyles,
        computed: &ComputedStyles,
        grid_templates: &GridTemplates,
        viewport: Size,
        scroll_offset: (f32, f32),
        paragraph: &ParagraphShaper,
        hyphenator: Option<&dyn Hyphenator>,
        editing: Option<EditingView<'_>>,
        cache: &MeasureCache,
        arrange_cache: &ArrangeCache,
    ) -> LayoutBoxes {
        let mut boxes = LayoutBoxes::default();
        let Some(root) = tree.root() else {
            return boxes;
        };
        // The root's containing block — and the fixed-positioning reference for
        // every `position: fixed` descendant — is the viewport rect at the origin.
        let viewport_rect = Rect::new(Point::ORIGIN, viewport);
        // The synthetic root *is* the initial containing block, and CSS 2.1 §10.1
        // says what that is: "The containing block … has the dimensions of the
        // viewport". So it is a box with a **specified** size, not one whose height
        // comes from its content — and saying so is not decoration. The synthetic
        // root has no computed style, so it would otherwise be laid out
        // `height: auto`; a flow box with an indefinite block size hands its
        // children an indefinite percentage basis, and every `height: 100%` in the
        // document would then behave as `auto`
        // (`specs/w3c/css-sizing-3.txt:554`) — correct for an author's auto-height
        // box and wrong for the ICB, which is the one box in the document whose
        // size the viewport decides.
        //
        // A tight constraint alone cannot carry this. `min == max` is also what an
        // arranging caller sends a box whose height it measured *from that box's
        // content*, so reading definiteness off the range would call a
        // content-derived height specified — the exact conflation
        // `BoxConstraints::percent_basis` exists to end.
        let icb = LayoutStyle {
            width: Dimension::Px(viewport.width),
            height: Dimension::Px(viewport.height),
            ..LayoutStyle::default()
        };
        cache.begin_pass();
        arrange_cache.begin_pass();
        arrange(
            tree,
            styles,
            computed,
            grid_templates,
            root,
            Point::ORIGIN,
            BoxConstraints::tight(viewport),
            &mut boxes,
            paragraph,
            hyphenator,
            viewport_rect,
            viewport_rect,
            scroll_offset,
            editing,
            MAX_DEPTH,
            &cache,
            arrange_cache,
            Some(icb),
        );
        // These boxes are what a later pass may serve an unchanged subtree from.
        arrange_cache.end_pass(&boxes);
        boxes
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;
    use obscur_layout::model::{
        AlignItems, FlexDirection, Inset, LayoutDisplay, MarginEdges, Position,
    };
    use obscur_style::computed::{ComputedStyle, ComputedValue};
    use obscur_style::property::{Length, PropertyId, PropertyValue, layout_keyword};
    use obscur_text::shaping::TextDirection;

    use crate::orchestration::text_leaf_measure::measure_face;
    use obscur_text::font::{FontId, MockFontResolver};
    use obscur_text::shaping::{
        CachingShaper, MockTextShaper, ScriptTag, SingleRunItemizer, TextItem, TextItemizer,
    };
    use obscur_text::span::TextRange;
    use obscur_tree::node::{LogicalParent, NodeKind};

    /// A default resolution viewport for the mapping tests.
    const TEST_VIEWPORT: Size = Size::new(800.0, 600.0);
    /// A default root font size for the mapping tests.
    const TEST_ROOT_FONT: f32 = 16.0;

    /// The default single-font paragraph orchestrator the layout tests measure
    /// through: a [`SingleRunItemizer`] + empty [`MockFontResolver`] + cached
    /// [`MockTextShaper`], reproducing the historical single-run behavior.
    fn test_paragraph() -> ParagraphShaper {
        ParagraphShaper::new(
            Box::new(SingleRunItemizer::new()),
            Box::new(MockFontResolver::new()),
            Box::new(CachingShaper::new(Box::new(MockTextShaper::new()))),
        )
    }

    /// A stub itemizer that cuts the fixed two-script string "abВГ" into a Latin
    /// run `[0..2)` and a Cyrillic run `[2..6)` (each Cyrillic char is 2 bytes).
    struct TwoScriptItemizer;

    impl TextItemizer for TwoScriptItemizer {
        fn itemize(&self, _text: &str, _base: TextDirection) -> Vec<TextItem> {
            vec![
                TextItem::new(
                    TextRange::new(0, 2),
                    ScriptTag::LATIN,
                    TextDirection::LeftToRight,
                    0,
                ),
                TextItem::new(
                    TextRange::new(2, 6),
                    ScriptTag::CYRILLIC,
                    TextDirection::LeftToRight,
                    0,
                ),
            ]
        }
    }

    /// A two-font paragraph orchestrator: the two-script itemizer plus a resolver
    /// mapping Latin→0 and Cyrillic→1, so "abВГ" shapes as two font-tagged runs.
    fn two_font_paragraph() -> ParagraphShaper {
        let mut resolver = MockFontResolver::new();
        resolver.register(ScriptTag::LATIN, FontId::new(0), measure_face());
        resolver.register(ScriptTag::CYRILLIC, FontId::new(1), measure_face());
        ParagraphShaper::new(
            Box::new(TwoScriptItemizer),
            Box::new(resolver),
            Box::new(CachingShaper::new(Box::new(MockTextShaper::new()))),
        )
    }

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
        let styles = LayoutStyles::default();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::default(),
            &GridTemplates::default(),
            Size::new(800.0, 600.0),
            (0.0, 0.0),
            &test_paragraph(),
        );
        assert!(boxes.is_empty());
    }

    #[test]
    fn layout_styles_from_maps_every_node() {
        let coordinator = LayoutCoordinator::new();
        let mut computed = ComputedStyles::default();
        let mut a = ComputedStyle::new();
        a.set(PropertyId::Width, ComputedValue::length(Length::px(10.0)));
        computed.insert(NodeId::from_raw(1), a);
        computed.insert(NodeId::from_raw(2), ComputedStyle::new());

        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
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

        let mut styles = LayoutStyles::default();
        styles.insert(root, block(200.0, 100.0));
        styles.insert(child_a, block(200.0, 50.0));
        styles.insert(child_b, block(200.0, 50.0));

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::default(),
            &GridTemplates::default(),
            Size::new(200.0, 100.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

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

        let mut styles = LayoutStyles::default();
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
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::default(),
            &GridTemplates::default(),
            Size::new(200.0, 100.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

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

        let mut styles = LayoutStyles::default();
        styles.insert(root, root_style);
        styles.insert(container, container_style);
        styles.insert(leaf, leaf_style);

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::default(),
            &GridTemplates::default(),
            Size::new(200.0, 200.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // Container offset by the root's top/left padding.
        assert_eq!(boxes[&container].origin(), Point::new(10.0, 10.0));
        // Leaf absolute origin = root padding (10) + container padding (5) = 15.
        assert_eq!(boxes[&leaf].origin(), Point::new(15.0, 15.0));
        assert_eq!(boxes[&leaf].size(), Size::new(50.0, 50.0));
    }

    #[test]
    fn relative_child_shifts_itself_and_subtree_without_moving_siblings() {
        // Block root (200x200) with two stacked block children `a` (relative) and
        // `b` (static). `a` holds a leaf grandchild so we can see the subtree shift.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 40);
        let b = add_child(&mut tree, root, 41);
        let leaf = add_child(&mut tree, a, 42);

        // `a` is `position: relative; top: 10px; left: 5px`.
        let a_relative = block(200.0, 50.0)
            .with_position(Position::Relative)
            .with_inset(Inset::new(
                Dimension::Px(10.0),
                Dimension::Auto,
                Dimension::Auto,
                Dimension::Px(5.0),
            ));

        let mut styles = LayoutStyles::default();
        styles.insert(root, block(200.0, 200.0));
        styles.insert(a, a_relative);
        styles.insert(b, block(200.0, 50.0));
        styles.insert(leaf, block(20.0, 20.0));

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::default(),
            &GridTemplates::default(),
            Size::new(200.0, 200.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // `a` is offset from its flow origin (0,0) by (left=5, top=10).
        assert_eq!(boxes[&a].origin(), Point::new(5.0, 10.0));
        // The leaf (flow-placed at a's content origin) shifts with the subtree.
        assert_eq!(boxes[&leaf].origin(), Point::new(5.0, 10.0));
        // The sibling `b` is untouched: still at the flow position (0, 50).
        assert_eq!(boxes[&b].origin(), Point::new(0.0, 50.0));
    }

    #[test]
    fn sticky_child_clamps_with_scroll_leaving_siblings_in_flow() {
        // Block root (200x1000): a 200x50 `spacer`, then a 200x50 `a` (sticky;
        // top:10) holding a leaf grandchild, then a 200x50 `b` (static). Flow:
        // spacer y=0, a y=50, b y=100. `a`'s static top (50) is below its 10px
        // inset, so at scroll 0 it sits at its static position (byte-identical).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let spacer = add_child(&mut tree, root, 49);
        let a = add_child(&mut tree, root, 50);
        let b = add_child(&mut tree, root, 51);
        let leaf = add_child(&mut tree, a, 52);

        let a_sticky = block(200.0, 50.0)
            .with_position(Position::Sticky)
            .with_inset(Inset::new(
                Dimension::Px(10.0),
                Dimension::Auto,
                Dimension::Auto,
                Dimension::Auto,
            ));

        let mut styles = LayoutStyles::default();
        styles.insert(root, block(200.0, 1000.0));
        styles.insert(spacer, block(200.0, 50.0));
        styles.insert(a, a_sticky);
        styles.insert(b, block(200.0, 50.0));
        styles.insert(leaf, block(20.0, 20.0));

        let coordinator = LayoutCoordinator::new();
        let viewport = Size::new(200.0, 1000.0);

        // scroll (0,0): sticky_lower = 0 + 10 = 10 <= static a.y = 50 → used = 50
        // (still below its threshold) → BYTE-IDENTICAL to static.
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::default(),
            &GridTemplates::default(),
            viewport,
            (0.0, 0.0),
            &test_paragraph(),
        );
        assert_eq!(
            boxes[&a].origin(),
            Point::new(0.0, 50.0),
            "scroll 0: a at static"
        );
        assert_eq!(boxes[&leaf].origin(), Point::new(0.0, 50.0));
        assert_eq!(
            boxes[&b].origin(),
            Point::new(0.0, 100.0),
            "sibling in flow"
        );

        // scroll (0,100): sticky_lower = 100 + 10 = 110 > static 50, clamp upper =
        // cb.bottom(1000) - h(50) = 950 → a sticks at y=110; its leaf rides along;
        // `b` stays at its unchanged flow position (0, 100).
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::default(),
            &GridTemplates::default(),
            viewport,
            (0.0, 100.0),
            &test_paragraph(),
        );
        assert_eq!(
            boxes[&a].origin(),
            Point::new(0.0, 110.0),
            "sticks at scrollport+inset"
        );
        assert_eq!(
            boxes[&leaf].origin(),
            Point::new(0.0, 110.0),
            "subtree rides along"
        );
        assert_eq!(
            boxes[&b].origin(),
            Point::new(0.0, 100.0),
            "sibling unmoved (in flow)"
        );

        // Huge scroll: sticky_lower far below cb → pinned at cb.bottom - h = 950.
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::default(),
            &GridTemplates::default(),
            viewport,
            (0.0, 5000.0),
            &test_paragraph(),
        );
        assert_eq!(
            boxes[&a].origin(),
            Point::new(0.0, 950.0),
            "pinned within CB"
        );
        assert_eq!(
            boxes[&b].origin(),
            Point::new(0.0, 100.0),
            "sibling still unmoved"
        );
    }

    #[test]
    fn static_child_is_not_offset_and_right_bottom_move_opposite() {
        // `a` static (no offset), `b` relative with `right`/`bottom` only.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 50);
        let b = add_child(&mut tree, root, 51);

        // `b` is `position: relative; right: 5px; bottom: 10px` -> moves (-5, -10).
        let b_relative = block(200.0, 50.0)
            .with_position(Position::Relative)
            .with_inset(Inset::new(
                Dimension::Auto,
                Dimension::Px(5.0),
                Dimension::Px(10.0),
                Dimension::Auto,
            ));

        let mut styles = LayoutStyles::default();
        styles.insert(root, block(200.0, 200.0));
        styles.insert(a, block(200.0, 50.0));
        styles.insert(b, b_relative);

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::default(),
            &GridTemplates::default(),
            Size::new(200.0, 200.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // Static `a` keeps its flow origin exactly.
        assert_eq!(boxes[&a].origin(), Point::new(0.0, 0.0));
        // `b` flow origin (0, 50) shifts by right=5 -> -5 x, bottom=10 -> -10 y.
        assert_eq!(boxes[&b].origin(), Point::new(-5.0, 40.0));
    }

    /// A block-only style with an explicit width and an auto (content-sized)
    /// height, for a container whose height must reflect only its in-flow content.
    fn auto_height_block(width: f32) -> LayoutStyle {
        LayoutStyle::default()
            .with_display(LayoutDisplay::Block)
            .with_width(Dimension::Px(width))
    }

    #[test]
    fn absolute_child_is_out_of_flow_and_positioned_by_insets() {
        // Scenario (a): a flow container with an in-flow `.a` and an absolute `.b`.
        // `.b` must not affect `.a` or the container's content height, and must be
        // placed at its insets within its containing block (the viewport here).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let container = add_child(&mut tree, root, 60);
        let a = add_child(&mut tree, container, 61);
        let b = add_child(&mut tree, container, 62);

        // `.b { position: absolute; top: 10; left: 20; width: 40; height: 30 }`.
        let b_absolute = block(40.0, 30.0)
            .with_position(Position::Absolute)
            .with_inset(Inset::new(
                Dimension::Px(10.0),
                Dimension::Auto,
                Dimension::Auto,
                Dimension::Px(20.0),
            ));

        let mut styles = LayoutStyles::default();
        styles.insert(root, block(300.0, 300.0));
        // Container is auto-height so its content height is observable.
        styles.insert(container, auto_height_block(300.0));
        styles.insert(a, block(300.0, 50.0));
        styles.insert(b, b_absolute);

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::default(),
            &GridTemplates::default(),
            Size::new(300.0, 300.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // `.a` is untouched by `.b`: flow origin (0, 0), its own size.
        assert_eq!(boxes[&a].origin(), Point::new(0.0, 0.0));
        assert_eq!(boxes[&a].size(), Size::new(300.0, 50.0));
        // The container's content height reflects only `.a` (50), not `.b`.
        assert_eq!(boxes[&container].size(), Size::new(300.0, 50.0));
        // `.b` is placed at its insets (20, 10) with its explicit 40x30 size.
        assert_eq!(boxes[&b].origin(), Point::new(20.0, 10.0));
        assert_eq!(boxes[&b].size(), Size::new(40.0, 30.0));
    }

    #[test]
    fn absolute_child_resolves_against_positioned_ancestor() {
        // Scenario (b): an absolute child positioned within a `position: relative`
        // ancestor (not the root), anchored by `right` / `bottom`.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let rel = add_child(&mut tree, root, 70);
        let abs = add_child(&mut tree, rel, 71);

        // `.rel { position: relative; left: 100; top: 100; width: 200; height: 200 }`.
        let rel_style = block(200.0, 200.0)
            .with_position(Position::Relative)
            .with_inset(Inset::new(
                Dimension::Px(100.0),
                Dimension::Auto,
                Dimension::Auto,
                Dimension::Px(100.0),
            ));
        // `.abs { position: absolute; right: 0; bottom: 0; width: 20; height: 20 }`.
        let abs_style = block(20.0, 20.0)
            .with_position(Position::Absolute)
            .with_inset(Inset::new(
                Dimension::Auto,
                Dimension::Px(0.0),
                Dimension::Px(0.0),
                Dimension::Auto,
            ));

        let mut styles = LayoutStyles::default();
        styles.insert(root, block(400.0, 400.0));
        styles.insert(rel, rel_style);
        styles.insert(abs, abs_style);

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::default(),
            &GridTemplates::default(),
            Size::new(400.0, 400.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // `.rel` is offset into the flow by its insets (100, 100).
        assert_eq!(boxes[&rel].origin(), Point::new(100.0, 100.0));
        // `.abs` sits flush against `.rel`'s far corner: (100+200-20, 100+200-20).
        assert_eq!(boxes[&abs].origin(), Point::new(280.0, 280.0));
        assert_eq!(boxes[&abs].size(), Size::new(20.0, 20.0));
    }

    #[test]
    fn absolute_child_with_both_insets_stretches_to_containing_block() {
        // Scenario (c): `left` and `right` both set pin both edges, so the width
        // stretches to the containing block regardless of the explicit width.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let abs = add_child(&mut tree, root, 80);

        // Explicit 999x999 is deliberately ignored on the pinned axes.
        let abs_style = block(999.0, 999.0)
            .with_position(Position::Absolute)
            .with_inset(Inset::new(
                Dimension::Px(0.0),
                Dimension::Px(30.0),
                Dimension::Px(0.0),
                Dimension::Px(10.0),
            ));

        let mut styles = LayoutStyles::default();
        styles.insert(root, block(300.0, 300.0));
        styles.insert(abs, abs_style);

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::default(),
            &GridTemplates::default(),
            Size::new(300.0, 300.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // Horizontal: x = 10, width = 300 - 10 - 30 = 260. Vertical: y = 0,
        // height = 300 - 0 - 0 = 300.
        assert_eq!(boxes[&abs].origin(), Point::new(10.0, 0.0));
        assert_eq!(boxes[&abs].size(), Size::new(260.0, 300.0));
    }

    #[test]
    fn fixed_child_positions_against_viewport_ignoring_ancestor() {
        // Scenario (d): a `position: fixed` child ignores its positioned ancestor
        // and resolves against the viewport, so `top: 0; left: 0` lands at (0, 0)
        // rather than the ancestor's (100, 100) origin.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let rel = add_child(&mut tree, root, 90);
        let fixed = add_child(&mut tree, rel, 91);

        let rel_style = block(200.0, 200.0)
            .with_position(Position::Relative)
            .with_inset(Inset::new(
                Dimension::Px(100.0),
                Dimension::Auto,
                Dimension::Auto,
                Dimension::Px(100.0),
            ));
        let fixed_style = block(20.0, 20.0)
            .with_position(Position::Fixed)
            .with_inset(Inset::new(
                Dimension::Px(0.0),
                Dimension::Auto,
                Dimension::Auto,
                Dimension::Px(0.0),
            ));

        let mut styles = LayoutStyles::default();
        styles.insert(root, block(400.0, 400.0));
        styles.insert(rel, rel_style);
        styles.insert(fixed, fixed_style);

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::default(),
            &GridTemplates::default(),
            Size::new(400.0, 400.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // `.rel` still sits at (100, 100), but the fixed child ignores it.
        assert_eq!(boxes[&rel].origin(), Point::new(100.0, 100.0));
        assert_eq!(boxes[&fixed].origin(), Point::new(0.0, 0.0));
        assert_eq!(boxes[&fixed].size(), Size::new(20.0, 20.0));
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

        let mut computed = ComputedStyles::default();

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
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        // Sanity: the bridge produced a flex container.
        assert_eq!(styles[&root].display, LayoutDisplay::Flex);

        let boxes = coordinator.layout(
            &tree,
            &styles,
            &computed,
            &GridTemplates::default(),
            Size::new(200.0, 100.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // Children sit side by side along the row axis: distinct x offsets.
        assert_eq!(boxes[&child_a].origin().x, 0.0);
        assert_eq!(boxes[&child_b].origin().x, 50.0);
        // Same y (a row, not a stack) — proving flex, not block, engaged.
        assert_eq!(boxes[&child_a].origin().y, boxes[&child_b].origin().y);
    }

    #[test]
    fn authored_grid_template_engages_two_columns() {
        use obscur_style::property::{GridTrack, PropertyValue};
        // A `display: grid` container with `grid-template-columns: 100px 50px`
        // and two children must place them side by side (col 0 then col 1). With
        // the default `GridLayout::new(1)` the two would stack in one column
        // (same x); the authored two-column template puts the second at x=100.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child_a = add_child(&mut tree, root, 50);
        let child_b = add_child(&mut tree, root, 51);

        let mut computed = ComputedStyles::default();
        let mut root_style = ComputedStyle::new();
        root_style.set(
            PropertyId::Display,
            ComputedValue::keyword(layout_keyword::DISPLAY_GRID),
        );
        root_style.set(PropertyId::Width, ComputedValue::length(Length::px(150.0)));
        root_style.set(PropertyId::Height, ComputedValue::length(Length::px(100.0)));
        root_style.set(
            PropertyId::GridTemplateColumns,
            ComputedValue::from_concrete(PropertyValue::grid_tracks([
                GridTrack::Fixed(Length::px(100.0)),
                GridTrack::Fixed(Length::px(50.0)),
            ]))
            .expect("concrete"),
        );
        computed.insert(root, root_style);
        computed.insert(child_a, ComputedStyle::new());
        computed.insert(child_b, ComputedStyle::new());

        let coordinator = LayoutCoordinator::new();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        let templates =
            coordinator.grid_templates_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert!(templates.contains_key(&root), "root authored a template");

        let boxes = coordinator.layout(
            &tree,
            &styles,
            &computed,
            &templates,
            Size::new(150.0, 100.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // First column at x=0 (100px wide), second column at x=100 (50px wide),
        // both on the same row — proving the two-column template engaged.
        assert_eq!(boxes[&child_a].origin().x, 0.0);
        assert_eq!(boxes[&child_a].size().width, 100.0);
        assert_eq!(boxes[&child_b].origin().x, 100.0);
        assert_eq!(boxes[&child_b].size().width, 50.0);
        assert_eq!(boxes[&child_a].origin().y, boxes[&child_b].origin().y);
    }

    /// One row of the external-consumer audit's 2-4 table, laid out through the
    /// whole coordinator exactly as the headless inspector measured it.
    ///
    /// The shell: `width/height: 100%`, `box-sizing: border-box`, `padding: 24px`,
    /// `display: grid`, `grid-template-columns: 1fr 1fr`, `gap: 10px`, in a
    /// 480×240 viewport — so the grid container's content box is 432 and each
    /// track is (432 − 10) / 2 = 211. One item, carrying `item_width`, with one
    /// child at `width: 50%`. Returns `(the item's used width, the child's)`.
    fn audit_grid_row(item_width: Length) -> (f32, f32) {
        use obscur_style::property::{GridTrack, PropertyValue};
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let item = add_child(&mut tree, root, 70);
        let grandchild = add_child(&mut tree, item, 71);

        let mut computed = ComputedStyles::default();
        let mut shell = ComputedStyle::new();
        shell.set(
            PropertyId::Display,
            ComputedValue::keyword(layout_keyword::DISPLAY_GRID),
        );
        shell.set(
            PropertyId::BoxSizing,
            ComputedValue::keyword(layout_keyword::BOX_SIZING_BORDER_BOX),
        );
        shell.set(
            PropertyId::Width,
            ComputedValue::length(Length::Percent(100.0)),
        );
        shell.set(
            PropertyId::Height,
            ComputedValue::length(Length::Percent(100.0)),
        );
        for edge in [
            PropertyId::PaddingTop,
            PropertyId::PaddingRight,
            PropertyId::PaddingBottom,
            PropertyId::PaddingLeft,
        ] {
            shell.set(edge, ComputedValue::length(Length::px(24.0)));
        }
        shell.set(PropertyId::Gap, ComputedValue::length(Length::px(10.0)));
        shell.set(
            PropertyId::GridTemplateColumns,
            ComputedValue::from_concrete(PropertyValue::grid_tracks([
                GridTrack::Fr(1.0),
                GridTrack::Fr(1.0),
            ]))
            .expect("concrete"),
        );
        computed.insert(root, shell);

        let mut item_style = ComputedStyle::new();
        item_style.set(PropertyId::Width, ComputedValue::length(item_width));
        computed.insert(item, item_style);

        let mut kid = ComputedStyle::new();
        kid.set(
            PropertyId::Width,
            ComputedValue::length(Length::Percent(50.0)),
        );
        computed.insert(grandchild, kid);

        let coordinator = LayoutCoordinator::new();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        let templates =
            coordinator.grid_templates_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &computed,
            &templates,
            Size::new(480.0, 240.0),
            (0.0, 0.0),
            &test_paragraph(),
        );
        (boxes[&item].size().width, boxes[&grandchild].size().width)
    }

    /// A grid item's children resolve their percentages against the **item's**
    /// box, not against the grid container's content box.
    ///
    /// `specs/w3c/css-grid-2.txt:712`, vendored here: "A grid item's grid area
    /// forms the containing block into which it is laid out." The arrange walk
    /// used to hand every child of a node the same percentage basis — that node's
    /// content box — which is the containing block for a block child and a flex
    /// item but not for a grid item. So in the `100%` row the item's own used
    /// width was right (211, its track) while its child measured **216**: the
    /// item's `100%` had been resolved against the grid container's 432, and that
    /// number — a containing block the item never had — is what the subtree
    /// inherited. The item's used size and the size its children saw disagreed.
    ///
    /// The `auto` row is the control: it never went through the item's own
    /// percentage, and it read 105.5 before this fix and reads 105.5 after.
    ///
    /// The `50px` row was the audit's **second**, separate defect (2-4 (b)): the
    /// item was stretched to its whole track (211) even though `stretch` applies
    /// only to a box whose size is `auto` (`specs/w3c/css-align-3.txt:1377`), and
    /// otherwise "falls back to flex-start". It now keeps the declared 50, and its
    /// child at 25 is half of *that* — the child of this row was always
    /// arithmetically consistent with the declared width, which is why the two
    /// defects had to be pinned apart before either was repaired: they fail in
    /// opposite directions and each could have masked the other.
    #[test]
    fn a_grid_items_children_resolve_against_the_item_not_the_grid_container() {
        assert_eq!(
            audit_grid_row(Length::Auto),
            (211.0, 105.5),
            "control: an `auto` item was always right"
        );
        assert_eq!(
            audit_grid_row(Length::Percent(100.0)),
            (211.0, 105.5),
            "the defect: the child measured 216 — 50% of the container's 432"
        );
        assert_eq!(
            audit_grid_row(Length::px(50.0)),
            (50.0, 25.0),
            "was (211, 25): `stretch` had run over the item's declared width"
        );
    }

    /// Allocates a text node carrying `content` under `parent`, wiring the child
    /// link and the logical parent, and returns its id.
    fn add_text_child(tree: &mut RetainedTree, parent: NodeId, content: &str) -> NodeId {
        let id = tree.alloc(NodeKind::Text {
            content: content.into(),
        });
        if let Some(node) = tree.get_mut(parent) {
            node.push_child(id);
        }
        if let Some(node) = tree.get_mut(id) {
            node.set_logical_parent(LogicalParent::of(parent));
        }
        id
    }

    /// Arranges a single text child under a fixed-size block `root` and returns
    /// the text node's arranged box. The text node carries no computed style, so
    /// its font size defaults to 16 (mock advance = 9.6 px/char, line_height 16).
    fn text_box_in_container(content: &str, width: f32, height: f32) -> LayoutBox {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, content);

        let mut computed = ComputedStyles::default();
        let mut root_style = ComputedStyle::new();
        root_style.set(PropertyId::Width, ComputedValue::length(Length::px(width)));
        root_style.set(
            PropertyId::Height,
            ComputedValue::length(Length::px(height)),
        );
        computed.insert(root, root_style);

        let coordinator = LayoutCoordinator::new();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &computed,
            &GridTemplates::default(),
            Size::new(width, height),
            (0.0, 0.0),
            &test_paragraph(),
        );
        boxes[&text]
    }

    /// Arranges a single text child through an explicit `paragraph` orchestrator
    /// and returns the text node's arranged box.
    fn text_box_with_paragraph(
        content: &str,
        width: f32,
        height: f32,
        paragraph: &ParagraphShaper,
    ) -> LayoutBox {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, content);

        let mut computed = ComputedStyles::default();
        let mut root_style = ComputedStyle::new();
        root_style.set(PropertyId::Width, ComputedValue::length(Length::px(width)));
        root_style.set(
            PropertyId::Height,
            ComputedValue::length(Length::px(height)),
        );
        computed.insert(root, root_style);

        let coordinator = LayoutCoordinator::new();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &computed,
            &GridTemplates::default(),
            Size::new(width, height),
            (0.0, 0.0),
            paragraph,
        );
        boxes[&text]
    }

    #[test]
    fn multi_font_text_measures_one_line_at_the_summed_run_width() {
        // "abВГ" itemizes into a Latin run and a Cyrillic run (two fonts). Each of
        // the 4 glyphs advances 9.6 (mock at 16px), so the wide-container measure
        // is one line: height == line_height (16) and width == 4·9.6 = 38.4. This
        // is the height the scene compose re-wraps to (paint line count agrees).
        let text_box = text_box_with_paragraph("abВГ", 1000.0, 200.0, &two_font_paragraph());
        assert_eq!(text_box.size().height, 16.0);
        assert!(
            (text_box.size().width - 38.4).abs() < 1e-3,
            "width {}",
            text_box.size().width
        );
    }

    #[test]
    fn long_text_in_a_narrow_container_wraps_to_multiple_lines() {
        // Requirement (a): three 4-char words (each 4·9.6 = 38.4 wide, plus a
        // trailing space → the first two words are 48.0 each) in a 50px-wide
        // container wrap to three lines, so the box is 3·line_height (48) tall —
        // strictly taller than a single line. Root is tall enough (200) that the
        // measured height is not clamped.
        let text_box = text_box_in_container("aaaa bbbb cccc", 50.0, 200.0);
        // Three lines stacked → height == 3 · 16.
        assert_eq!(text_box.size().height, 48.0);
        // Widest line is "aaaa " but its trailing space now HANGS (CSS Text 3
        // §3/§7.1), so the used width is 4·9.6 = 38.4 (was 5·9.6 = 48 when the
        // trailing space wrongly counted) — still within the 50px container.
        assert!(
            (text_box.size().width - 38.4).abs() < 1e-3,
            "width {}",
            text_box.size().width
        );
    }

    #[test]
    fn line_height_two_scales_the_wrapped_box_height() {
        // The same three-word string in a 50px container still wraps to three
        // lines, but `line-height: 2` makes each line box 2·16 = 32 tall, so the
        // measured box is 3·32 = 96 — twice the default 48 — reserving the extra
        // vertical space so following content does not overlap.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "aaaa bbbb cccc");

        let mut computed = ComputedStyles::default();
        let mut root_style = ComputedStyle::new();
        root_style.set(PropertyId::Width, ComputedValue::length(Length::px(50.0)));
        root_style.set(PropertyId::Height, ComputedValue::length(Length::px(300.0)));
        computed.insert(root, root_style);
        let mut text_style = ComputedStyle::new();
        text_style.set(PropertyId::LineHeight, ComputedValue::number(2.0));
        computed.insert(text, text_style);

        let coordinator = LayoutCoordinator::new();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &computed,
            &GridTemplates::default(),
            Size::new(50.0, 300.0),
            (0.0, 0.0),
            &test_paragraph(),
        );
        // Three wrapped lines at a 32px line box → 96px tall.
        assert_eq!(boxes[&text].size().height, 96.0);
    }

    #[test]
    fn same_text_in_a_wide_container_stays_one_line() {
        // Requirement (b): the identical string in a 1000px container is a single
        // line — one line_height tall — and as wide as the whole run.
        let text_box = text_box_in_container("aaaa bbbb cccc", 1000.0, 200.0);
        assert_eq!(text_box.size().height, 16.0);
        // Full run advance: 14 chars · 9.6 = 134.4.
        assert!(
            (text_box.size().width - 134.4).abs() < 1e-3,
            "width {}",
            text_box.size().width
        );
    }

    #[test]
    fn short_single_line_text_is_sized_to_its_run() {
        // Requirement (e): short text that fits stays one line at its run advance.
        let text_box = text_box_in_container("hi", 1000.0, 200.0);
        assert_eq!(text_box.size().height, 16.0);
        // 2 chars · 9.6 = 19.2.
        assert!(
            (text_box.size().width - 19.2).abs() < 1e-3,
            "width {}",
            text_box.size().width
        );
    }

    #[test]
    fn empty_text_yields_one_empty_line_and_does_not_panic() {
        // Requirement (d): an empty string measures to a zero-width, one-line box
        // (height == line_height) without panicking.
        let text_box = text_box_in_container("", 100.0, 200.0);
        assert_eq!(text_box.size().width, 0.0);
        assert_eq!(text_box.size().height, 16.0);
    }

    #[test]
    fn text_font_size_drives_the_wrapped_box() {
        // The node's computed font-size (20px → advance 12.0, line_height 20)
        // drives the shaped/wrapped size, not the 16px default.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "abcd");

        let mut computed = ComputedStyles::default();
        let mut root_style = ComputedStyle::new();
        root_style.set(PropertyId::Width, ComputedValue::length(Length::px(1000.0)));
        root_style.set(PropertyId::Height, ComputedValue::length(Length::px(200.0)));
        computed.insert(root, root_style);
        let mut text_computed = ComputedStyle::new();
        text_computed.set(
            PropertyId::FontSize,
            ComputedValue::length(Length::px(20.0)),
        );
        computed.insert(text, text_computed);

        let coordinator = LayoutCoordinator::new();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(styles[&text].font_size, 20.0);
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &computed,
            &GridTemplates::default(),
            Size::new(1000.0, 200.0),
            (0.0, 0.0),
            &test_paragraph(),
        );
        let text_box = boxes[&text];
        // 4 chars · (20 · 0.6) = 48.0 wide, one line 20 tall.
        assert!(
            (text_box.size().width - 48.0).abs() < 1e-3,
            "width {}",
            text_box.size().width
        );
        assert_eq!(text_box.size().height, 20.0);
    }

    #[test]
    fn display_none_generates_no_box_for_subtree() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let mut styles = LayoutStyles::default();
        styles.insert(
            root,
            LayoutStyle::default().with_display(LayoutDisplay::None),
        );

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::default(),
            &GridTemplates::default(),
            Size::new(100.0, 100.0),
            (0.0, 0.0),
            &test_paragraph(),
        );
        // A `None` root produces no box at all.
        assert!(boxes.is_empty());
    }

    #[test]
    fn nested_column_flex_item_measured_at_full_intrinsic_height() {
        // Regression (the wireframe "cramping"): a flex ROW whose item is itself a
        // flex COLUMN must reserve the column's FULL stacked height (control slot +
        // margin + caption), so consecutive rows do not overlap.
        //
        //   root(block) -> row(flex row, align flex-start)
        //     -> col(flex column, width 264)
        //          -> slot(flex row) -> control(block 100x28)
        //          -> cap(block, height 14)
        //
        // The column stacks a 28px slot (with a 12px bottom margin) over a 14px
        // caption, so its intrinsic height is 28 + 12 + 14 = 54; the row reserves
        // exactly that and the caption sits at y = 40..54, clear of the slot.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let row = add_child(&mut tree, root, 100);
        let col = add_child(&mut tree, row, 101);
        let slot = add_child(&mut tree, col, 102);
        let control = add_child(&mut tree, slot, 103);
        let cap = add_child(&mut tree, col, 104);

        let mut flex_row_start = LayoutStyle::default().with_display(LayoutDisplay::Flex);
        flex_row_start.flex_direction = FlexDirection::Row;
        flex_row_start.align_items = AlignItems::FlexStart;

        let mut col_style = LayoutStyle::default()
            .with_display(LayoutDisplay::Flex)
            .with_width(Dimension::Px(264.0));
        col_style.flex_direction = FlexDirection::Column;

        let mut slot_style = flex_row_start.clone();
        slot_style.margin = MarginEdges::px(0.0, 0.0, 12.0, 0.0);

        let cap_style = LayoutStyle::default()
            .with_display(LayoutDisplay::Block)
            .with_height(Dimension::Px(14.0));

        let mut styles = LayoutStyles::default();
        styles.insert(
            root,
            LayoutStyle::default().with_display(LayoutDisplay::Block),
        );
        styles.insert(row, flex_row_start);
        styles.insert(col, col_style);
        styles.insert(slot, slot_style);
        styles.insert(control, block(100.0, 28.0));
        styles.insert(cap, cap_style);

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::default(),
            &GridTemplates::default(),
            Size::new(1200.0, 4160.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // The column child is measured at its full intrinsic height (54), and the
        // row reserves that — NOT the ~28px slot-only height that caused overlap.
        assert_eq!(boxes[&col].size().height, 54.0);
        assert_eq!(boxes[&row].size().height, 54.0);
        // Within the column the caption sits fully BELOW the slot: slot 0..28,
        // caption 40..54, no overlap.
        assert_eq!(boxes[&slot].origin().y, 0.0);
        assert_eq!(boxes[&slot].size().height, 28.0);
        assert_eq!(boxes[&cap].origin().y, 40.0);
        assert_eq!(boxes[&cap].size().height, 14.0);
    }

    #[test]
    fn placeholder_slot_is_measured_from_the_session_text_not_the_empty_value() {
        // The measure↔compose invariant for the CSS `::placeholder` slot: an
        // empty-valued field's ONLY text node is the `.placeholder` overlay. While a
        // session with typed text is active, BOTH passes shape the SESSION buffer over
        // that slot (`editing_override`), so the measured box is sized from the session
        // text — matching the glyphs the compose pass paints there (no measure/paint
        // skew). An empty session leaves the slot sized from the muted placeholder hint
        // (`:placeholder-shown` holds). Single-line mock: 9.6 px/char, so the two cases
        // differ purely by string length.
        use obscur_text::selection::TextSelection;
        let build = || {
            let mut tree = RetainedTree::new();
            let root = tree.alloc(NodeKind::Root);
            tree.set_root(root);
            let field = add_child(&mut tree, root, 700);
            let overlay = add_child(&mut tree, field, 701);
            // Mark the overlay element as the `.placeholder` pseudo-element.
            {
                use obscur_view::view::{CLASS, StyleAttributes, ViewProperty, ViewValue};
                let attrs = StyleAttributes::extract(&[ViewProperty::new(
                    CLASS,
                    ViewValue::Text("placeholder".into()),
                )]);
                if let Some(n) = tree.get_mut(overlay) {
                    n.set_attributes(attrs);
                }
            }
            let hint = add_text_child(&mut tree, overlay, "x"); // a 1-char muted hint
            let mut computed = ComputedStyles::default();
            computed.insert(root, ComputedStyle::new());
            computed.insert(field, ComputedStyle::new());
            computed.insert(overlay, ComputedStyle::new());
            computed.insert(hint, ComputedStyle::new());
            (tree, computed, field, hint)
        };
        let coordinator = LayoutCoordinator::new();
        let advance = 16.0 * 0.6; // mock advance per char

        // Typed session "abcd" over the placeholder → the slot is sized from "abcd".
        let (tree, computed, field, hint) = build();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        let typed = coordinator.layout_editing(
            &tree,
            &styles,
            &computed,
            &GridTemplates::default(),
            Size::new(400.0, 200.0),
            (0.0, 0.0),
            &test_paragraph(),
            None,
            Some(EditingView::new(field, "abcd", TextSelection::caret(4))),
        );
        assert!(
            (typed[&hint].size().width - advance * 4.0).abs() < 0.5,
            "placeholder slot measured from the 4-char session, got {}",
            typed[&hint].size().width,
        );

        // Empty session → the slot stays sized from the 1-char placeholder hint.
        let (tree, computed, field, hint) = build();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        let empty = coordinator.layout_editing(
            &tree,
            &styles,
            &computed,
            &GridTemplates::default(),
            Size::new(400.0, 200.0),
            (0.0, 0.0),
            &test_paragraph(),
            None,
            Some(EditingView::new(field, "", TextSelection::caret(0))),
        );
        assert!(
            (empty[&hint].size().width - advance).abs() < 0.5,
            "empty session leaves the slot sized from the 1-char placeholder hint, got {}",
            empty[&hint].size().width,
        );
    }

    // ---- §3.4 step 8: incremental layout ----------------------------------

    /// `root` (a 400x300 block) > `wrapper` > three text siblings.
    ///
    /// The wrapper exists because `arrange` drives the root's algorithm directly
    /// and never routes it through [`measure`] — so the root never appears in the
    /// cache at all, and an *upward* propagation claim needs a real measured
    /// ancestor to be about.
    ///
    /// `wrapper_size` gives the wrapper an explicit width/height; [`None`] leaves
    /// it auto-sized, so its box grows with its content.
    ///
    /// Returns the tree, the computed styles, and `(root, wrapper, a, b, c)`.
    fn flow_of_three_texts(
        wrapper_size: Option<(f32, f32)>,
    ) -> (RetainedTree, ComputedStyles, [NodeId; 5]) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let wrapper = add_child(&mut tree, root, 1);
        let a = add_text_child(&mut tree, wrapper, "alpha");
        let b = add_text_child(&mut tree, wrapper, "bravo");
        let c = add_text_child(&mut tree, wrapper, "charlie");

        let sized = |width: f32, height: f32| {
            let mut style = ComputedStyle::new();
            style.set(PropertyId::Width, ComputedValue::length(Length::px(width)));
            style.set(
                PropertyId::Height,
                ComputedValue::length(Length::px(height)),
            );
            style
        };
        let mut computed = ComputedStyles::default();
        computed.insert(root, sized(400.0, 300.0));
        computed.insert(
            wrapper,
            match wrapper_size {
                Some((width, height)) => sized(width, height),
                None => ComputedStyle::new(),
            },
        );
        for node in [a, b, c] {
            computed.insert(node, ComputedStyle::new());
        }
        (tree, computed, [root, wrapper, a, b, c])
    }

    /// The pair of caches a runtime carries across frames. `Default::default()` is
    /// "nothing retained", i.e. a full pass.
    #[derive(Default)]
    struct Caches {
        /// Retained `measure(node, constraints)` sizes.
        measures: MeasureCache,
        /// Retained `arrange` keys and boxes.
        arranges: ArrangeCache,
    }

    /// Lays `tree` out through `caches`, re-deriving the layout styles each call
    /// exactly as the runtime does.
    fn lay_out(
        tree: &RetainedTree,
        computed: &ComputedStyles,
        paragraph: &ParagraphShaper,
        caches: &Caches,
        dirty: Option<&LayoutInvalidation>,
    ) -> LayoutBoxes {
        lay_out_scrolled(tree, computed, paragraph, caches, dirty, (0.0, 0.0))
    }

    /// [`lay_out`] at an explicit viewport scroll offset.
    fn lay_out_scrolled(
        tree: &RetainedTree,
        computed: &ComputedStyles,
        paragraph: &ParagraphShaper,
        caches: &Caches,
        dirty: Option<&LayoutInvalidation>,
        scroll: (f32, f32),
    ) -> LayoutBoxes {
        let coordinator = LayoutCoordinator::new();
        let styles = coordinator.layout_styles_from(computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        coordinator.layout_invalidated(
            tree,
            &styles,
            computed,
            &GridTemplates::default(),
            Size::new(400.0, 300.0),
            scroll,
            paragraph,
            None,
            None,
            &caches.measures,
            &caches.arranges,
            dirty,
        )
    }

    #[test]
    fn a_scrolled_frame_reuses_every_measure_and_moves_no_box() {
        // The half of the scroll claim that lives here: given the classifier no
        // longer degrades a scroll-only frame to `Full` (which passes
        // `dirty = None`, and `None` is what CLEARS both caches wholesale), the
        // coordinator must actually be able to serve that frame from what it
        // retained. Two things have to hold, and neither is obvious:
        //
        //  * no box moves — layout is in document coordinates, and the offset
        //    reaches the walk only through `Inset::sticky_offset`;
        //  * every text leaf's measure is a cache HIT — the measure key is
        //    `(node, constraints)` and carries no scroll, so re-descending the
        //    tree (the arrange key DOES carry scroll) re-shapes nothing.
        //
        // Without the classifier fix this frame arrives with `dirty = None`, both
        // caches are emptied, and all three text leaves are re-shaped from
        // scratch — at 60 Hz, for every pixel of a flick.
        let (tree, computed, [_root, wrapper, a, b, c]) = flow_of_three_texts(Some((400.0, 300.0)));
        let paragraph = test_paragraph();
        let caches = Caches::default();

        let at_rest = lay_out_scrolled(&tree, &computed, &paragraph, &caches, None, (0.0, 0.0));
        let retained = caches.measures.len();
        assert!(retained > 0, "frame 1 must retain some measures");
        for node in [wrapper, a, b, c] {
            assert!(caches.measures.measured(node), "frame 1 measures {node:?}");
        }

        // Frame 2: 40 pixels of scroll and nothing else, so the dirty set the
        // classifier produces is empty — it names no node whose box can move.
        let unchanged = LayoutInvalidation::new();
        let scrolled = lay_out_scrolled(
            &tree,
            &computed,
            &paragraph,
            &caches,
            Some(&unchanged),
            (0.0, 40.0),
        );

        assert_eq!(
            caches.measures.measured_count(),
            0,
            "a scrolled frame must re-measure NOTHING; it re-measured {} node(s)",
            caches.measures.measured_count(),
        );
        assert_eq!(
            caches.measures.len(),
            retained,
            "a scrolled frame must not evict a single retained measure"
        );
        assert_eq!(
            at_rest, scrolled,
            "no `position: sticky` box here, so scrolling moves nothing"
        );
    }

    #[test]
    fn incremental_layout_remeasures_only_the_dirty_node_and_its_ancestors() {
        // The wrapper is explicitly sized, which is what makes this subtree one
        // "whose size provably cannot change its parent's": §4.2's invalidation
        // boundary. An auto-sized container instead re-sizes with its content, and
        // its new box is a new constraint for every child — see
        // `an_auto_container_reoffers_constraints_to_its_whole_flow`.
        let (tree, mut computed, [_root, wrapper, a, b, c]) =
            flow_of_three_texts(Some((400.0, 300.0)));
        let paragraph = test_paragraph();
        let caches = Caches::default();

        // Frame 1: nothing retained, so everything is measured.
        lay_out(&tree, &computed, &paragraph, &caches, None);
        for node in [wrapper, a, b, c] {
            assert!(
                caches.measures.measured(node),
                "frame 1 must measure {node:?}"
            );
        }

        // Frame 2: the LAST sibling's own font-size changes, and the frame says so.
        // Last, because a flow hands each child a `remaining_height` net of its
        // preceding siblings — so only a change to the last one leaves every other
        // child's constraints untouched. See
        // `a_size_change_remeasures_the_later_siblings_it_moves`.
        computed.get_mut(&c).expect("styled").set(
            PropertyId::FontSize,
            ComputedValue::length(Length::px(32.0)),
        );
        let mut dirty = LayoutInvalidation::new();
        dirty.note_style_changed(c);
        lay_out(&tree, &computed, &paragraph, &caches, Some(&dirty));

        // The changed node re-measures...
        assert!(
            caches.measures.measured(c),
            "the dirty node must re-measure"
        );
        // ...and so does its container: §4.3's upward propagation. `c` grew, and that
        // is a contribution to `wrapper`'s content size.
        assert!(
            caches.measures.measured(wrapper),
            "the dirty node's ancestor must re-measure — its child's size changed",
        );
        // But the clean siblings do NOT. This is the whole claim: `a` and `b` are
        // handed the same constraints they were handed last frame, and nothing they
        // read changed, so their sizes are served from the cache and their text is
        // never re-shaped.
        assert!(
            !caches.measures.measured(a),
            "a clean sibling must not re-measure"
        );
        assert!(
            !caches.measures.measured(b),
            "a clean sibling must not re-measure"
        );
        assert!(
            caches.measures.measured_count() <= 4,
            "only the dirty node and its one ancestor, each under at most its probe \
             and arrange constraints — got {}",
            caches.measures.measured_count(),
        );
    }

    #[test]
    fn an_auto_container_reoffers_constraints_to_its_whole_flow() {
        // The honest other half, pinned rather than left to surprise someone.
        //
        // `FlowLayout` hands each child
        // `remaining_height = avail_content_height - (preceding heights)`, and an
        // auto-sized container's `avail_content_height` is its own box — which grew.
        // So growing ANY child of an auto container genuinely changes the
        // constraints every child is measured under, later siblings and earlier ones
        // alike. §4.3 propagates down "only when constraints change" — and here they
        // DO change, so these re-measures are required, not wasted: a cache hit
        // would be a stale box, not a saving.
        //
        // Collapsing them needs a per-context `LayoutDependency` recording that a
        // text leaf's size ignores the block-axis constraint
        // (`LayoutContext::should_remeasure`). Nothing populates that yet, so this
        // is exactly where the incremental pass stops.
        let (tree, mut computed, [_root, _wrapper, a, b, c]) = flow_of_three_texts(None);
        let paragraph = test_paragraph();
        let caches = Caches::default();
        lay_out(&tree, &computed, &paragraph, &caches, None);

        computed.get_mut(&a).expect("styled").set(
            PropertyId::FontSize,
            ComputedValue::length(Length::px(32.0)),
        );
        let mut dirty = LayoutInvalidation::new();
        dirty.note_style_changed(a);
        lay_out(&tree, &computed, &paragraph, &caches, Some(&dirty));

        assert!(caches.measures.measured(a), "the dirty node re-measures");
        assert!(
            caches.measures.measured(b) && caches.measures.measured(c),
            "an auto container's grown box re-offers every child a new constraint",
        );
    }

    /// A [`Hyphenator`] that proposes no break (identical to [`NullHyphenator`])
    /// but counts how many times the wrap pass consulted it.
    ///
    /// This is the probe for work the [`MeasureCache`] cannot see: it is called
    /// from inside [`measure_text_node`]'s wrap, on whatever path reaches it. So a
    /// pass that re-wraps a text leaf increments it whether it went through
    /// [`measure`] or shaped the leaf itself.
    #[derive(Debug, Default)]
    struct CountingHyphenator {
        calls: std::cell::Cell<usize>,
    }

    impl Hyphenator for CountingHyphenator {
        fn hyphenate(&self, _word: &str) -> Vec<usize> {
            self.calls.set(self.calls.get() + 1);
            Vec::new()
        }
    }

    #[test]
    fn the_arrange_walk_sizes_text_through_the_cache_too() {
        // `arrange` used to carry a byte-identical copy of `measure`'s text branch,
        // so it re-shaped and re-wrapped every text leaf on every frame no matter
        // what the cache held — the cache saved the *probe* and nothing else.
        //
        // The hyphenator counts the wrap itself, which is the work at stake:
        // paragraph shaping is already memoized by `ParagraphShaper`, but
        // `apply_spacing` / `apply_tab_stops` / `wrap_strut_hyphenated` are not, and
        // they run per text leaf per frame.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        // A word far wider than the container, so the wrap must consult the
        // hyphenator; the hyphenator proposes nothing, so the boxes are unchanged.
        let text = add_text_child(&mut tree, root, "supercalifragilisticexpialidocious");

        let mut computed = ComputedStyles::default();
        let mut root_style = ComputedStyle::new();
        root_style.set(PropertyId::Width, ComputedValue::length(Length::px(100.0)));
        root_style.set(PropertyId::Height, ComputedValue::length(Length::px(200.0)));
        computed.insert(root, root_style);
        computed.insert(text, ComputedStyle::new());

        let paragraph = test_paragraph();
        let hyphenator = CountingHyphenator::default();
        let coordinator = LayoutCoordinator::new();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        let lay_out = |caches: &Caches, dirty: Option<&LayoutInvalidation>| {
            coordinator.layout_invalidated(
                &tree,
                &styles,
                &computed,
                &GridTemplates::default(),
                Size::new(100.0, 200.0),
                (0.0, 0.0),
                &paragraph,
                Some(&hyphenator),
                None,
                &caches.measures,
                &caches.arranges,
                dirty,
            )
        };

        let caches = Caches::default();
        let first = lay_out(&caches, None);
        let after_first = hyphenator.calls.get();
        assert!(
            after_first > 0,
            "the fixture must actually reach the wrap, or this test proves nothing",
        );

        // Frame 2: nothing changed at all. Not one line box may be re-wrapped.
        let second = lay_out(&caches, Some(&LayoutInvalidation::new()));
        assert_eq!(
            hyphenator.calls.get(),
            after_first,
            "an unchanged frame re-wrapped text: the arrange walk is not going \
             through the measure cache",
        );
        assert_eq!(first, second, "and it must still produce the same boxes");
        assert_eq!(
            caches.measures.measured_count(),
            0,
            "nothing was measured at all"
        );
    }

    #[test]
    fn incremental_layout_agrees_with_a_full_pass_box_for_box() {
        // The reuse is only worth anything if it is INDISTINGUISHABLE from the full
        // pass. A sibling's size moves every later sibling in a flow, so this also
        // pins that the arrange half still propagates: `a` growing must push `b`
        // and `c` down even though neither re-measured.
        let (tree, mut computed, [_root, _wrapper, a, ..]) = flow_of_three_texts(None);
        let paragraph = test_paragraph();

        let caches = Caches::default();
        lay_out(&tree, &computed, &paragraph, &caches, None);
        let before = lay_out(&tree, &computed, &paragraph, &Caches::default(), None);

        computed.get_mut(&a).expect("styled").set(
            PropertyId::FontSize,
            ComputedValue::length(Length::px(32.0)),
        );
        let mut dirty = LayoutInvalidation::new();
        dirty.note_style_changed(a);

        let incremental = lay_out(&tree, &computed, &paragraph, &caches, Some(&dirty));
        let full = lay_out(&tree, &computed, &paragraph, &Caches::default(), None);

        assert_eq!(
            incremental, full,
            "an incremental pass must produce the full pass's boxes exactly",
        );
        assert_ne!(
            incremental, before,
            "the change must actually have moved something, or this proves nothing",
        );
    }

    #[test]
    fn a_dirty_root_drops_the_whole_cache() {
        // The root's `font-size` is the `rem` base every node's lengths resolve
        // against, so no retained size survives it.
        let (tree, computed, [root, wrapper, a, b, c]) = flow_of_three_texts(None);
        let paragraph = test_paragraph();
        let caches = Caches::default();
        lay_out(&tree, &computed, &paragraph, &caches, None);

        let mut dirty = LayoutInvalidation::new();
        dirty.note_style_changed(root);
        assert!(
            invalidated_layout(&tree, &dirty).is_none(),
            "a dirty root cannot be bounded",
        );

        lay_out(&tree, &computed, &paragraph, &caches, Some(&dirty));
        for node in [wrapper, a, b, c] {
            assert!(caches.measures.measured(node), "{node:?} must re-measure");
        }
    }

    #[test]
    fn an_unreachable_dirty_node_drops_the_whole_cache() {
        // A node the walk cannot reach from the root has no known ancestor chain,
        // so the set of measures it can have changed is unknown.
        let (tree, ..) = flow_of_three_texts(None);
        let mut dirty = LayoutInvalidation::new();
        dirty.note_style_changed(NodeId::from_raw(9_999));
        assert!(invalidated_layout(&tree, &dirty).is_none());
    }

    #[test]
    fn a_dirty_container_evicts_its_text_children() {
        // A text node's measure reads its CONTAINER's computed `text-overflow`
        // (see `overflow_for_text`), so a dirty container re-measures its children
        // even though nothing of theirs changed.
        let (mut tree, _computed, [root, wrapper, a, ..]) = flow_of_three_texts(None);
        // A second, unrelated container so there is something that must survive.
        let other = add_child(&mut tree, root, 2);
        let other_text = add_text_child(&mut tree, other, "elsewhere");

        let mut dirty = LayoutInvalidation::new();
        dirty.note_style_changed(wrapper);
        let evicted = invalidated_layout(&tree, &dirty)
            .expect("boundable")
            .measures;
        assert!(evicted.contains(&wrapper), "the dirty node itself");
        assert!(evicted.contains(&root), "its ancestor");
        assert!(
            evicted.contains(&a),
            "its text child reads its `text-overflow`",
        );
        assert!(
            !evicted.contains(&other) && !evicted.contains(&other_text),
            "an unrelated sibling subtree must survive",
        );
    }

    // ---- §3.4 step 8: incremental ARRANGE ---------------------------------
    //
    // Every test below counts `ArrangeCache::arranged` rather than inspecting the
    // boxes, because "the boxes are in the right place" is equally true of a full
    // arrange and proves nothing about incrementality. The boxes are then checked
    // against a full pass as well, because an arrange that skips the wrong subtree
    // is not slow, it is *wrong*.

    #[test]
    fn an_unchanged_frame_rearranges_nothing() {
        let (tree, computed, _) = flow_of_three_texts(None);
        let paragraph = test_paragraph();

        let caches = Caches::default();
        let first = lay_out(&tree, &computed, &paragraph, &caches, None);
        assert!(
            caches.arranges.arranged_count() > 0,
            "frame 1 must actually arrange something, or this test proves nothing",
        );

        // Frame 2: nothing changed at all. Not one node may be re-arranged.
        let second = lay_out(
            &tree,
            &computed,
            &paragraph,
            &caches,
            Some(&LayoutInvalidation::new()),
        );
        assert_eq!(
            caches.arranges.arranged_count(),
            0,
            "an unchanged frame re-arranged a node: `arrange` is still O(nodes)",
        );
        assert_eq!(first, second, "and it must still produce the same boxes");

        // Frame 3, the induction step: frame 2 produced its map entirely by COPYING
        // retained boxes, and that map is what frame 3 must now serve from. A pass
        // that ends with anything less than the full map would break here and
        // nowhere else.
        let third = lay_out(
            &tree,
            &computed,
            &paragraph,
            &caches,
            Some(&LayoutInvalidation::new()),
        );
        assert_eq!(
            caches.arranges.arranged_count(),
            0,
            "a served frame's own output must itself be servable",
        );
        assert_eq!(first, third, "and it must still produce the same boxes");
    }

    #[test]
    fn a_dirty_leaf_rearranges_its_ancestors_but_not_the_siblings_above_it() {
        // `c` is LAST in the flow, so `a` and `b` sit above it: nothing `c` does to
        // its own size can move them.
        let (tree, mut computed, [root, wrapper, a, b, c]) = flow_of_three_texts(None);
        let paragraph = test_paragraph();

        let caches = Caches::default();
        lay_out(&tree, &computed, &paragraph, &caches, None);

        computed.get_mut(&c).expect("styled").set(
            PropertyId::FontSize,
            ComputedValue::length(Length::px(32.0)),
        );
        let mut dirty = LayoutInvalidation::new();
        dirty.note_style_changed(c);
        lay_out(&tree, &computed, &paragraph, &caches, Some(&dirty));

        assert!(caches.arranges.arranged(c), "the dirty node itself");
        assert!(
            caches.arranges.arranged(wrapper),
            "its parent must re-run its flow: that is how `c`'s new box is derived",
        );
        assert!(caches.arranges.arranged(root), "and the root above it");
        assert!(
            !caches.arranges.arranged(a),
            "`a` sits above the change and cannot have moved: it must be served",
        );
        assert!(
            !caches.arranges.arranged(b),
            "`b` sits above the change and cannot have moved: it must be served",
        );
        assert_eq!(
            caches.arranges.arranged_count(),
            3,
            "exactly the dirty node and its ancestor chain re-arranged",
        );
    }

    #[test]
    fn a_sibling_pushed_down_by_a_grown_node_rearranges_though_it_is_clean() {
        // THE hazard. `arrange` writes ABSOLUTE positions, so growing `a` moves `b`
        // and `c` down — and neither is dirty, neither re-measures, and neither has
        // any style change at all. A cache that skipped every non-dirty node would
        // leave them painted at last frame's `y`. What must catch them is their
        // recomputed `origin` missing in the `ArrangeKey`.
        let (tree, mut computed, [_root, _wrapper, a, b, c]) = flow_of_three_texts(None);
        let paragraph = test_paragraph();

        let caches = Caches::default();
        let before = lay_out(&tree, &computed, &paragraph, &caches, None);

        computed.get_mut(&a).expect("styled").set(
            PropertyId::FontSize,
            ComputedValue::length(Length::px(32.0)),
        );
        let mut dirty = LayoutInvalidation::new();
        dirty.note_style_changed(a);
        let incremental = lay_out(&tree, &computed, &paragraph, &caches, Some(&dirty));

        // The premise: they really did move, and really were not re-measured. If
        // either of these fails the assertions below are vacuous.
        assert_ne!(
            before.get(&b),
            incremental.get(&b),
            "`a` growing must actually move `b`, or this test proves nothing",
        );
        // And the sharp end of it: `b` and `c` are NOT in the dirty set's reach.
        // An arrange gated on the dirty set alone — however carefully that set is
        // propagated — would serve both of them and paint them at last frame's `y`.
        let evicted = invalidated_layout(&tree, &dirty).expect("boundable");
        assert!(
            !evicted.arranges.contains(&b) && !evicted.arranges.contains(&c),
            "the dirty set must NOT cover the moved siblings, or this test is really \
             about the dirty set and proves nothing about the key",
        );

        assert!(
            caches.arranges.arranged(b),
            "`b` was pushed down by `a`: it must re-arrange despite being clean",
        );
        assert!(
            caches.arranges.arranged(c),
            "`c` was pushed down too: it must re-arrange despite being clean",
        );

        let full = lay_out(&tree, &computed, &paragraph, &Caches::default(), None);
        assert_eq!(
            incremental, full,
            "an incremental arrange must produce the full pass's boxes exactly",
        );
    }

    #[test]
    fn a_restyle_that_only_moves_children_inside_an_unchanged_box_still_rearranges_them() {
        // The mirror of the test above, and the case the `ArrangeKey` alone CANNOT
        // catch: the flex container is explicitly sized, so flipping its
        // `justify-content` moves its item without changing one bit of the
        // container's own key — same origin, same constraints, same containing
        // block. Only the dirty set can know its subtree must be rebuilt.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let flexbox = add_child(&mut tree, root, 1);
        let item = add_child(&mut tree, flexbox, 2);

        let sized = |width: f32, height: f32| {
            let mut style = ComputedStyle::new();
            style.set(PropertyId::Width, ComputedValue::length(Length::px(width)));
            style.set(
                PropertyId::Height,
                ComputedValue::length(Length::px(height)),
            );
            style
        };
        let mut computed = ComputedStyles::default();
        computed.insert(root, sized(400.0, 300.0));
        let mut flex_style = sized(400.0, 100.0);
        flex_style.set(
            PropertyId::Display,
            ComputedValue::from_concrete(PropertyValue::Keyword(layout_keyword::DISPLAY_FLEX))
                .expect("concrete"),
        );
        computed.insert(flexbox, flex_style);
        computed.insert(item, sized(50.0, 50.0));

        let paragraph = test_paragraph();
        let caches = Caches::default();
        let before = lay_out(&tree, &computed, &paragraph, &caches, None);

        computed.get_mut(&flexbox).expect("styled").set(
            PropertyId::JustifyContent,
            ComputedValue::from_concrete(PropertyValue::Keyword(layout_keyword::JUSTIFY_END))
                .expect("concrete"),
        );
        let mut dirty = LayoutInvalidation::new();
        dirty.note_style_changed(flexbox);
        let incremental = lay_out(&tree, &computed, &paragraph, &caches, Some(&dirty));

        // The premise: the container's own box is byte-identical, so its key is too.
        assert_eq!(
            before.get(&flexbox),
            incremental.get(&flexbox),
            "the container's own box must NOT change, or this tests the key rather \
             than the dirty set",
        );
        assert_ne!(
            before.get(&item),
            incremental.get(&item),
            "the item must actually move to the end, or this test proves nothing",
        );

        let full = lay_out(&tree, &computed, &paragraph, &Caches::default(), None);
        assert_eq!(
            incremental, full,
            "a restyled container's children must land where a full pass puts them",
        );
    }

    #[test]
    fn a_clean_subtree_beside_a_dirty_one_is_served_whole() {
        // `wrapper` is EXPLICITLY sized, which is what makes `other` provably
        // stable: a change inside a fixed-size box cannot move what follows it. So
        // `other`'s whole subtree — not just `other` — must be served, which is the
        // saving `arrange` incrementality exists for.
        let (mut tree, mut computed, [root, _wrapper, a, ..]) =
            flow_of_three_texts(Some((400.0, 100.0)));
        let other = add_child(&mut tree, root, 2);
        let deep = add_child(&mut tree, other, 3);
        let deeper = add_text_child(&mut tree, deep, "elsewhere");
        for node in [other, deep, deeper] {
            computed.insert(node, ComputedStyle::new());
        }

        let paragraph = test_paragraph();
        let caches = Caches::default();
        let before = lay_out(&tree, &computed, &paragraph, &caches, None);
        assert!(
            before.contains_key(&deeper),
            "the fixture must actually arrange the deep subtree, or the skip below \
             is skipping nothing",
        );

        computed.get_mut(&a).expect("styled").set(
            PropertyId::FontSize,
            ComputedValue::length(Length::px(32.0)),
        );
        let mut dirty = LayoutInvalidation::new();
        dirty.note_style_changed(a);
        let incremental = lay_out(&tree, &computed, &paragraph, &caches, Some(&dirty));

        assert!(
            caches.arranges.arranged(a),
            "the dirty node itself re-arranges"
        );
        for node in [other, deep, deeper] {
            assert!(
                !caches.arranges.arranged(node),
                "{node:?} is behind a fixed-size sibling and cannot have moved: the \
                 whole subtree must be served, not walked",
            );
        }

        let full = lay_out(&tree, &computed, &paragraph, &Caches::default(), None);
        assert_eq!(
            incremental, full,
            "the served subtree's retained boxes must be exactly what a full pass \
             would have written",
        );
    }
}
