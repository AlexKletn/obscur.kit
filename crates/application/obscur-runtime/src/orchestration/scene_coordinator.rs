//! Composes a paint-ordered display list from the styled + laid-out tree:
//! [`SceneCoordinator`].

use std::collections::HashSet;

use obscur_identity::NodeId;
use obscur_scene::color::Color;
use obscur_scene::command::{DisplayList, DisplayListBuilder};
use obscur_scene::paint::Brush;
use obscur_style::computed::ComputedValue;
use obscur_style::property::{PropertyId, StyleColor};
use obscur_tree::tree::RetainedTree;

use super::{ComputedStyles, LayoutBoxes};

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
/// Phase 1 emits only **background fills**: a node whose computed
/// `background-color` is present and not fully transparent fills its arranged
/// layout rectangle. Opacity/clip layers, borders, text, and images are
/// deferred to later phases (they will bracket the child fills with
/// push/pop layer/clip pairs). Because Phase 1 emits only leaf
/// [`fill_rect`](DisplayListBuilder::fill_rect) commands, the resulting
/// [`DisplayList`] is always well-formed (balanced) and can be consumed by a
/// renderer or `Scene` directly.
///
/// The coordinator is a stateless zero-sized value; construct it with
/// [`new`](Self::new) or [`Default`].
#[derive(Debug, Clone, Copy, Default)]
pub struct SceneCoordinator;

impl SceneCoordinator {
    /// Creates a coordinator.
    #[must_use]
    pub const fn new() -> Self {
        Self
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
                if color.a > 0.0 {
                    if let Some(layout_box) = boxes.get(&node) {
                        builder
                            .fill_rect(layout_box.bounds, Brush::solid(scene_color_from(color)));
                    }
                }
            }

            if let Some(retained) = tree.get(node) {
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

    /// Builds a `ComputedStyle` carrying a single `background-color`.
    fn with_background(color: StyleColor) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::BackgroundColor,
            ComputedValue::color(color),
        );
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
        assert_eq!(rects[0], Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 100.0)));
        assert_eq!(rects[1], Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 50.0)));
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
    fn scene_color_from_copies_components() {
        let style_color = StyleColor::rgba(0.25, 0.5, 0.75, 0.9);
        let color = scene_color_from(style_color);
        assert_eq!(color, Color::rgba(0.25, 0.5, 0.75, 0.9));
    }
}
