//! Composes a paint-ordered display list from the styled + laid-out tree:
//! [`SceneCoordinator`].

use std::collections::HashSet;

use obscur_identity::NodeId;
use obscur_scene::color::Color;
use obscur_scene::command::{DisplayList, DisplayListBuilder};
use obscur_scene::paint::Brush;
use obscur_scene::text::{GlyphRun, PositionedGlyph};
use obscur_style::computed::ComputedValue;
use obscur_style::property::{PropertyId, StyleColor};
use obscur_text::font::{FontAttributes, FontFace, FontFamily, FontMetrics, FontSource, GenericFamily};
use obscur_text::shaping::{MockTextShaper, ShapingRequest, TextShaper};
use obscur_tree::tree::RetainedTree;

use super::{ComputedStyles, LayoutBoxes};

/// Default font size (in pixels) used to emit text runs.
///
/// The fixed [`PropertyId`] set carries no font-size property yet, so text runs
/// are emitted at this default. When a font-size property is introduced this
/// constant becomes the fallback and the computed value takes precedence.
const DEFAULT_FONT_SIZE: f32 = 16.0;

/// Orchestrates the scene phase: turns the styled + laid-out tree into a
/// Scene/Paint [`DisplayList`] with a deterministic paint order (spec §5.1,
/// §3.10, §3.4).
///
/// The coordinator is **orchestration only**: it walks the retained tree and
/// emits paint commands through `obscur-scene`'s [`DisplayListBuilder`], but it
/// performs no rasterization — turning the display list into pixels is a
/// renderer concern in a later phase.
///
/// # Paint order
///
/// The tree is walked **pre-order** (a parent is emitted before its children,
/// children left-to-right), which is exactly the scene's deterministic paint
/// order: a parent paints first and its children paint on top (spec §5.1). The
/// walk is **cycle-safe** — a visited-set guard means a shared or cyclic child
/// link terminates instead of looping — and **panic-free**: every lookup is
/// fallible and no indexing is performed.
///
/// # Phase-1 scope
///
/// The coordinator emits **background fills** and **shaped text runs**: a node
/// whose computed `background-color` is present and not fully transparent fills
/// its arranged layout rectangle, and a text node with an arranged box emits a
/// [`draw_text`](DisplayListBuilder::draw_text) run at that box's origin.
///
/// Text is **shaped once, here in the pipeline**, through an `obscur-text`
/// [`TextShaper`] port (the deterministic [`MockTextShaper`] by default): the
/// coordinator turns the node's string into a [`GlyphRun`] of positioned glyphs
/// and carries those glyphs in the [`DrawText`](obscur_scene::command::PaintCommand::DrawText)
/// command, so every renderer consumes an identical run and only rasterization
/// differs (spec §5.10, §5.2). A font-size property does not exist yet, so runs
/// use [`DEFAULT_FONT_SIZE`]. Opacity/clip layers, borders, and images are
/// deferred to later phases. Because only leaf
/// [`fill_rect`](DisplayListBuilder::fill_rect) and
/// [`draw_text`](DisplayListBuilder::draw_text) commands are emitted, the
/// resulting [`DisplayList`] is always well-formed (balanced) and can be consumed
/// by a renderer or `Scene` directly.
///
/// The coordinator owns its shaper and a default resolved face; construct it with
/// [`new`](Self::new) or [`Default`].
#[derive(Debug, Clone)]
pub struct SceneCoordinator {
    /// The shaper turning text-node strings into positioned glyphs. The default
    /// [`MockTextShaper`] is deterministic: identical input yields identical
    /// glyphs, which is what the cross-backend determinism boundary relies on.
    shaper: MockTextShaper,
    /// The resolved face requests are shaped against. The mock shaper is
    /// font-agnostic (it needs no real metrics), so a generic monospace face
    /// suffices as the default; a real pipeline would resolve this per node via a
    /// `FontProvider`.
    default_face: FontFace,
}

impl SceneCoordinator {
    /// Creates a coordinator with the deterministic [`MockTextShaper`] and a
    /// generic monospace default face.
    #[must_use]
    pub fn new() -> Self {
        Self {
            shaper: MockTextShaper::new(),
            default_face: FontFace::new(
                FontFamily::generic(GenericFamily::Monospace),
                FontAttributes::default(),
                FontSource::system("mock"),
                FontMetrics::new(1000, 800.0, -200.0, 0.0),
            ),
        }
    }

    /// Shapes `content` at [`DEFAULT_FONT_SIZE`] into a backend-independent
    /// [`GlyphRun`] of positioned glyphs.
    ///
    /// Shaping runs **once**, here: the `obscur-text` shaper produces glyphs with
    /// per-glyph advances, which this method lays out left-to-right into pen
    /// positions relative to the run origin (the baseline sits [`DEFAULT_FONT_SIZE`]
    /// below the top-left, matching the box's ascent). The resulting run is what
    /// both the GPU and software backends consume verbatim.
    fn shape_run(&self, content: &str) -> GlyphRun {
        let request = ShapingRequest::new(content, self.default_face.clone(), DEFAULT_FONT_SIZE);
        let shaped = self.shaper.shape(&request);
        let mut pen_x = 0.0_f32;
        let glyphs: Vec<PositionedGlyph> = shaped
            .glyphs()
            .iter()
            .map(|glyph| {
                let positioned = PositionedGlyph::new(
                    glyph.glyph_id(),
                    glyph.cluster(),
                    pen_x + glyph.x_offset(),
                    DEFAULT_FONT_SIZE + glyph.y_offset(),
                    glyph.x_advance(),
                );
                pen_x += glyph.x_advance();
                positioned
            })
            .collect();
        GlyphRun::new(glyphs, DEFAULT_FONT_SIZE, content)
    }

    /// Composes a paint-ordered [`DisplayList`] of background fills from the
    /// styled + laid-out tree.
    ///
    /// Returns an empty (and balanced) display list when the tree has no root.
    /// Otherwise the tree is walked pre-order from the root (parent before
    /// children, children left-to-right, cycle-safe); for each visited node that
    /// has a non-transparent computed `background-color` **and** an arranged box
    /// in `boxes`, a fill of that box's rectangle is appended in paint order.
    ///
    /// `computed` supplies each node's [`ComputedStyle`](obscur_style::computed::ComputedStyle)
    /// (typically produced by the style coordinator) and `boxes` supplies each
    /// node's absolute layout rectangle (produced by the layout coordinator). A
    /// node missing from either map simply contributes no fill.
    #[must_use]
    pub fn compose(
        &self,
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
    ) -> DisplayList {
        let mut builder = DisplayListBuilder::new();
        let Some(root) = tree.root() else {
            return builder.build();
        };

        // Pre-order walk (parent before children). Children are pushed reversed
        // so the leftmost child is popped first, preserving left-to-right paint
        // order. The visited set makes the walk cycle-safe and panic-free.
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut stack: Vec<NodeId> = vec![root];
        while let Some(node) = stack.pop() {
            if !visited.insert(node) {
                // Already painted (cycle or shared child): do not revisit.
                continue;
            }

            if let Some(color) = background_color_of(computed, node) {
                // Skip fully transparent backgrounds: they paint nothing.
                if color.a > 0.0
                    && let Some(layout_box) = boxes.get(&node)
                {
                    builder.fill_rect(layout_box.bounds, Brush::solid(scene_color_from(color)));
                }
            }

            if let Some(retained) = tree.get(node) {
                // Text nodes emit a text run at their arranged box origin. A node
                // is either a text node or an element with a background, so this
                // never double-emits with the background fill above. A text node
                // may have a zero-size box; the run is still emitted (the renderer
                // decides its extent). A text node with no arranged box is skipped.
                if let Some(content) = retained.kind().text()
                    && let Some(layout_box) = boxes.get(&node)
                {
                    let color = text_color_of(computed, node);
                    let run = self.shape_run(content);
                    builder.draw_text(layout_box.origin(), run, Brush::solid(color));
                }

                for &child in retained.children().iter().rev() {
                    if !visited.contains(&child) {
                        stack.push(child);
                    }
                }
            }
        }

        builder.build()
    }
}

impl Default for SceneCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Reads a node's computed `background-color`, if present and color-valued.
///
/// Returns [`None`] when the node has no computed style, no `background-color`
/// property, or a non-color value for it.
fn background_color_of(computed: &ComputedStyles, node: NodeId) -> Option<StyleColor> {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::BackgroundColor))
        .and_then(ComputedValue::as_color)
}

/// Reads a node's computed text `color` as a scene [`Color`], defaulting to
/// [`Color::WHITE`] when absent.
///
/// Mirrors [`background_color_of`] but reads [`PropertyId::Color`]. When the node
/// has no computed style, no `color` property, or a non-color value, the run
/// falls back to opaque white (visible on the example's colored bars).
fn text_color_of(computed: &ComputedStyles, node: NodeId) -> Color {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::Color))
        .and_then(ComputedValue::as_color)
        .map_or(Color::WHITE, scene_color_from)
}

/// Converts an authored style color into a scene paint color.
///
/// [`StyleColor`] is the authored style color (straight, non-premultiplied sRGB
/// components); the scene [`Color`] is the paint color a renderer consumes.
/// Phase 1 copies the components verbatim — the linear/premultiplied conversion
/// the GPU needs is a renderer concern applied later, not in the coordinator.
fn scene_color_from(c: StyleColor) -> Color {
    Color::rgba(c.r, c.g, c.b, c.a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{Point, Rect, Size};
    use obscur_identity::ComponentId;
    use obscur_layout::model::LayoutBox;
    use obscur_scene::command::PaintCommand;
    use obscur_style::computed::ComputedStyle;
    use obscur_tree::node::{LogicalParent, NodeKind};

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

    /// Builds a `ComputedStyle` carrying a single text `color`.
    fn with_color(color: StyleColor) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::Color, ComputedValue::color(color));
        style
    }

    /// Collects the `DrawText` runs (origin, glyph run, brush) in a display list.
    fn text_runs(list: &DisplayList) -> Vec<(Point, GlyphRun, Brush)> {
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawText { origin, run, brush } => {
                    Some((*origin, run.clone(), brush.clone()))
                }
                _ => None,
            })
            .collect()
    }

    /// Builds a `ComputedStyle` carrying a single `background-color`.
    fn with_background(color: StyleColor) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::BackgroundColor, ComputedValue::color(color));
        style
    }

    /// A layout box at the origin with the given size.
    fn box_at(x: f32, y: f32, w: f32, h: f32) -> LayoutBox {
        LayoutBox::new(Rect::new(Point::new(x, y), Size::new(w, h)))
    }

    /// Counts the `DrawPath` (fill) commands in a display list.
    fn fill_count(list: &DisplayList) -> usize {
        list.commands()
            .iter()
            .filter(|c| matches!(c, PaintCommand::DrawPath { .. }))
            .count()
    }

    #[test]
    fn empty_tree_composes_empty_balanced_list() {
        let coordinator = SceneCoordinator::new();
        let tree = RetainedTree::new();
        let computed = ComputedStyles::new();
        let boxes = LayoutBoxes::new();

        let list = coordinator.compose(&tree, &computed, &boxes);
        assert!(list.is_empty());
        assert!(list.is_balanced());
    }

    #[test]
    fn root_and_two_children_fill_in_preorder() {
        // root (bg) -> [child_a (bg), child_b (no bg)].
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child_a = add_child(&mut tree, root, 10);
        let child_b = add_child(&mut tree, root, 11);

        let root_bg = StyleColor::rgba(1.0, 0.0, 0.0, 1.0);
        let child_a_bg = StyleColor::rgba(0.0, 1.0, 0.0, 1.0);

        let mut computed = ComputedStyles::new();
        computed.insert(root, with_background(root_bg));
        computed.insert(child_a, with_background(child_a_bg));
        // child_b has a style but no background-color.
        computed.insert(child_b, ComputedStyle::new());

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 100.0));
        boxes.insert(child_a, box_at(0.0, 0.0, 100.0, 50.0));
        boxes.insert(child_b, box_at(0.0, 50.0, 100.0, 50.0));

        let coordinator = SceneCoordinator::new();
        let list = coordinator.compose(&tree, &computed, &boxes);

        // Two nodes have a non-transparent background -> two fills.
        assert_eq!(fill_count(&list), 2);
        assert_eq!(list.len(), 2);
        assert!(list.is_balanced());

        // Paint order is pre-order: the root's fill is emitted first, then the
        // child's. Assert the rectangles in order.
        let rects: Vec<Rect> = list
            .commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { path, .. } => path.bounds(),
                _ => None,
            })
            .collect();
        assert_eq!(rects.len(), 2);
        assert_eq!(
            rects[0],
            Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 100.0))
        );
        assert_eq!(
            rects[1],
            Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 50.0))
        );
    }

    #[test]
    fn fully_transparent_background_emits_no_fill() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let transparent = StyleColor::rgba(1.0, 0.0, 0.0, 0.0);
        let mut computed = ComputedStyles::new();
        computed.insert(root, with_background(transparent));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 10.0, 10.0));

        let coordinator = SceneCoordinator::new();
        let list = coordinator.compose(&tree, &computed, &boxes);

        assert!(list.is_empty());
        assert!(list.is_balanced());
    }

    #[test]
    fn node_without_a_box_emits_no_fill() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let mut computed = ComputedStyles::new();
        computed.insert(root, with_background(StyleColor::rgba(0.0, 0.0, 1.0, 1.0)));

        // No arranged box supplied for the root.
        let boxes = LayoutBoxes::new();

        let coordinator = SceneCoordinator::new();
        let list = coordinator.compose(&tree, &computed, &boxes);
        assert!(list.is_empty());
    }

    #[test]
    fn text_node_emits_run_at_box_origin_with_default_white() {
        // root (bg) -> text child (no color set -> default white).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "hello");

        let mut computed = ComputedStyles::new();
        computed.insert(root, with_background(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)));
        // Text node has no computed color property.
        computed.insert(text, ComputedStyle::new());

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 100.0));
        boxes.insert(text, box_at(5.0, 12.0, 40.0, 16.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes);

        // Background element still fills.
        assert_eq!(fill_count(&list), 1);

        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].0, Point::new(5.0, 12.0));
        // The run was shaped in the domain: five chars -> five positioned glyphs.
        assert_eq!(runs[0].1.text(), "hello");
        assert_eq!(runs[0].1.len(), 5);
        assert_eq!(runs[0].2, Brush::solid(Color::WHITE));

        // Leaf-only list stays balanced.
        assert!(list.is_balanced());
    }

    #[test]
    fn text_node_shapes_one_positioned_glyph_per_char_in_order() {
        // A four-char run yields four glyphs, laid out left-to-right at the
        // mock's fixed advance, on the baseline (DEFAULT_FONT_SIZE below origin).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "abcd");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, 40.0, 16.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes);
        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1);
        let run = &runs[0].1;
        assert_eq!(run.len(), 4);
        let advance = DEFAULT_FONT_SIZE * 0.6;
        for (i, glyph) in run.glyphs().iter().enumerate() {
            assert_eq!(glyph.glyph_id(), u32::from(['a', 'b', 'c', 'd'][i]));
            assert_eq!(glyph.cluster(), i);
            assert_eq!(glyph.x(), advance * i as f32);
            assert_eq!(glyph.y(), DEFAULT_FONT_SIZE);
            assert_eq!(glyph.advance(), advance);
        }
    }

    #[test]
    fn text_shaping_is_deterministic_across_composes() {
        // The same input tree must produce byte-identical positioned glyphs on
        // every compose (the property both backends' agreement rests on).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "Count: 7");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(2.0, 3.0, 80.0, 16.0));

        let coordinator = SceneCoordinator::new();
        let first = text_runs(&coordinator.compose(&tree, &computed, &boxes));
        let second = text_runs(&coordinator.compose(&tree, &computed, &boxes));
        assert_eq!(first, second);
    }

    #[test]
    fn text_node_uses_its_computed_color() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "tinted");

        let color = StyleColor::rgba(0.1, 0.2, 0.3, 1.0);
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(text, with_color(color));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(1.0, 2.0, 10.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes);

        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].2, Brush::solid(Color::rgba(0.1, 0.2, 0.3, 1.0)));
    }

    #[test]
    fn zero_size_text_box_still_emits_a_run() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "z");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());

        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(3.0, 4.0, 0.0, 0.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes);

        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].0, Point::new(3.0, 4.0));
        assert_eq!(runs[0].1.text(), "z");
        assert_eq!(runs[0].1.len(), 1);
    }

    #[test]
    fn text_node_without_a_box_emits_no_run() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "orphan");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());

        // No arranged box for the text node.
        let boxes = LayoutBoxes::new();

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes);
        assert!(text_runs(&list).is_empty());
        assert!(list.is_empty());
    }

    #[test]
    fn scene_color_from_copies_components() {
        let style_color = StyleColor::rgba(0.25, 0.5, 0.75, 0.9);
        let color = scene_color_from(style_color);
        assert_eq!(color, Color::rgba(0.25, 0.5, 0.75, 0.9));
    }
}
