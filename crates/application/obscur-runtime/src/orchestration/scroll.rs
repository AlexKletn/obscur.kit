#![doc = include_str!("../../docs/api/orchestration/scroll.doc/_module.md")]

use std::collections::HashSet;

use obscur_geometry::{Rect, Size};
use obscur_identity::{FastBuildHasher, FastSet, NodeId};
use obscur_style::computed::OverflowAxes;
use obscur_tree::tree::RetainedTree;

use super::{ComputedStyles, LayoutBoxes};

/// A resolved scroll container: a node whose computed `overflow` is `scroll` or
/// `auto` **and** whose scrollable content is taller than its scrollport, so it
/// owns a vertical scroll position (CSS Overflow 3 §scroll-container).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ScrollContainer {
    /// The scroll-container element.
    pub node: NodeId,
    /// The scrollport rect (logical px). This implementation uses the **border
    /// box** — matching the existing `overflow` paint clip — where CSS Overflow 3
    /// defines the scrollport as the padding box: a documented simplification kept
    /// deliberately in step with the coordinator's overflow clip so the clipped
    /// region and the scrolled region coincide exactly.
    pub scrollport: Rect,
    /// The scrollable content height (logical px): the bottom-most descendant edge
    /// measured from the scrollport top (CSS Overflow 3 §scrollable-overflow-region),
    /// never less than the scrollport height.
    pub content_height: f32,
    /// Scrollable content width in logical pixels.
    pub content_width: f32,
    /// Whether the horizontal scroll origin is on the right edge.
    pub rtl: bool,
    /// The node's tree depth (root = 0), so a consumer can resolve nested
    /// containers outermost-first.
    pub depth: u32,
}

impl ScrollContainer {
    /// Length of the horizontal scroll range in logical pixels.
    pub(crate) fn horizontal_range(self) -> f32 {
        (self.content_width - self.scrollport.size.width).max(0.0)
    }

    /// Clamp a signed horizontal scroll offset to the container’s reachable range.
    pub(crate) fn clamp_left(self, offset: f32) -> f32 {
        let range = self.horizontal_range();
        if self.rtl {
            offset.clamp(-range, 0.0)
        } else {
            offset.clamp(0.0, range)
        }
    }

    /// The maximum `scroll_top` (logical px): `content_height − scrollport_height`,
    /// clamped non-negative — the scroll position is clamped to the scrollable
    /// overflow rectangle (CSS Overflow 3 §scrolling).
    #[must_use]
    pub(crate) fn max_scroll_top(self) -> f32 {
        (self.content_height - self.scrollport.size.height).max(0.0)
    }
}

#[doc = include_str!("../../docs/api/orchestration/scroll.doc/overflow_scrolls.md")]
#[must_use]
pub(crate) fn overflow_scrolls(computed: &ComputedStyles, node: NodeId) -> bool {
    OverflowAxes::of(computed.get(&node)).block_scrolls()
}

pub(crate) fn overflow_scrolls_x(computed: &ComputedStyles, node: NodeId) -> bool {
    use obscur_style::property::layout_keyword::{OVERFLOW_AUTO, OVERFLOW_SCROLL};
    matches!(
        OverflowAxes::of(computed.get(&node)).inline_axis(),
        OVERFLOW_AUTO | OVERFLOW_SCROLL
    )
}

#[doc = include_str!("../../docs/api/orchestration/scroll.doc/establishes_scroll_container.md")]
#[must_use]
pub(crate) fn establishes_scroll_container(computed: &ComputedStyles, node: NodeId) -> bool {
    OverflowAxes::of(computed.get(&node)).establishes_scroll_container()
}

/// Width and height of the reachable overflow, bounded below by the scrollport. Horizontal RTL measures overflow to the left. Nested scroll containers contribute their own box.
#[must_use]
fn scrollable_content_size(
    tree: &RetainedTree,
    computed: &ComputedStyles,
    boxes: &LayoutBoxes,
    node: NodeId,
    scrollport: Rect,
    rtl: bool,
) -> Size {
    let mut min_left = scrollport.origin.x;
    let mut max_right = scrollport.right();
    let mut max_bottom = scrollport.bottom();
    let mut stack: Vec<NodeId> = tree.visual_children(node).collect();
    let mut visited: FastSet<NodeId> = HashSet::default();
    while let Some(child) = stack.pop() {
        if !visited.insert(child) {
            continue;
        }
        if let Some(b) = boxes.get(&child) {
            min_left = min_left.min(b.bounds.origin.x);
            max_right = max_right.max(b.bounds.right());
            max_bottom = max_bottom.max(b.bounds.bottom());
        }
        // Do not descend into a nested scroll container: it clips its own overflow.
        if !establishes_scroll_container(computed, child) {
            stack.extend(tree.visual_children(child));
        }
    }
    Size::new(
        if rtl {
            scrollport.right() - min_left
        } else {
            max_right - scrollport.origin.x
        },
        max_bottom - scrollport.origin.y,
    )
}

#[doc = include_str!("../../docs/api/orchestration/scroll.doc/scroll_containers.md")]
#[must_use]
pub(crate) fn scroll_containers(
    tree: &RetainedTree,
    computed: &ComputedStyles,
    boxes: &LayoutBoxes,
) -> Vec<ScrollContainer> {
    let Some(root) = tree.root() else {
        return Vec::new();
    };
    let mut out: Vec<ScrollContainer> = Vec::new();
    let mut stack: Vec<(NodeId, u32)> = vec![(root, 0)];
    // This walk touches every live node, twice per frame, whether or not the
    // document has a single scroll container; reserving the set is the cheap half
    // of that (the walk itself is §3.4 K8's other half, and untouched here).
    let mut visited: FastSet<NodeId> =
        HashSet::with_capacity_and_hasher(tree.len(), FastBuildHasher::default());
    while let Some((node, depth)) = stack.pop() {
        if !visited.insert(node) {
            continue;
        }
        if establishes_scroll_container(computed, node)
            && let Some(b) = boxes.get(&node)
        {
            let scrollport = super::overflow_padding_box(computed.get(&node), b.bounds);
            use super::layout_style_mapping::{direction_of, writing_mode_of};
            let rtl = computed.get(&node).is_some_and(|style| {
                !writing_mode_of(style).is_vertical() && direction_of(style).is_rtl()
            });
            let extent = scrollable_content_size(tree, computed, boxes, node, scrollport, rtl);
            use obscur_style::property::layout_keyword::{OVERFLOW_CLIP, OVERFLOW_VISIBLE};
            let axes = OverflowAxes::of(computed.get(&node));
            let content_width = if matches!(axes.inline_axis(), OVERFLOW_VISIBLE | OVERFLOW_CLIP) {
                scrollport.size.width
            } else {
                extent.width
            };
            let content_height = if matches!(axes.block_axis(), OVERFLOW_VISIBLE | OVERFLOW_CLIP) {
                scrollport.size.height
            } else {
                extent.height
            };
            if content_height > scrollport.size.height || content_width > scrollport.size.width {
                out.push(ScrollContainer {
                    node,
                    scrollport,
                    content_height,
                    content_width,
                    rtl,
                    depth,
                });
            }
        }
        for child in tree.visual_children(node) {
            stack.push((child, depth.saturating_add(1)));
        }
    }
    out.sort_by_key(|c| c.depth);
    out
}
