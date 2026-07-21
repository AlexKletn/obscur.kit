//! Per-element vertical scroll containers: which nodes scroll, their scrollport,
//! and their scrollable content extent.
//!
//! Reimplemented from **CSS Overflow Module Level 3** — a box whose computed
//! `overflow` is `scroll` or `auto` is a **scroll container** (§overflow) with a
//! **scrollport** (the visible cutout its content is clipped and scrolled within,
//! §scrollable-overflow) and a **scrollable overflow region** (the union of its
//! content's overflow, §scrollable-overflow-region). This module is pure and
//! IO-free: the same tree/style/box inputs always yield the same containers, so
//! both the runtime (which owns the live scroll position + wheel + hit-test) and
//! the scene coordinator (which paints the scroll offset + scrollbar) resolve the
//! identical set from one source of truth.

use std::collections::HashSet;

use obscur_geometry::Rect;
use obscur_identity::NodeId;
use obscur_style::computed::ComputedValue;
use obscur_style::property::{PropertyId, PropertyValue, layout_keyword};
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
    /// The node's tree depth (root = 0), so a consumer can resolve nested
    /// containers outermost-first.
    pub depth: u32,
}

impl ScrollContainer {
    /// The maximum `scroll_top` (logical px): `content_height − scrollport_height`,
    /// clamped non-negative — the scroll position is clamped to the scrollable
    /// overflow rectangle (CSS Overflow 3 §scrolling).
    #[must_use]
    pub(crate) fn max_scroll_top(self) -> f32 {
        (self.content_height - self.scrollport.size.height).max(0.0)
    }
}

/// Whether a node's computed `overflow` makes it a **scroll container** — the
/// reserved `scroll` or `auto` handles (CSS Overflow 3 §overflow). `visible` /
/// `hidden` / `clip` / absent are not scroll containers (they never own a scroll
/// position, even though `hidden` / `clip` still clip).
#[must_use]
pub(crate) fn overflow_scrolls(computed: &ComputedStyles, node: NodeId) -> bool {
    matches!(
        computed
            .get(&node)
            .and_then(|style| style.get(PropertyId::Overflow))
            .map(ComputedValue::value),
        Some(&PropertyValue::Keyword(sym))
            if sym == layout_keyword::OVERFLOW_SCROLL || sym == layout_keyword::OVERFLOW_AUTO
    )
}

/// Whether a node's computed `overflow` makes it a **scroll container** in the
/// CSS Overflow 3 §overflow sense: `scroll`, `auto`, **or `hidden`**. Unlike
/// [`overflow_scrolls`] (which names only the visibly-scrollable `scroll`/`auto`),
/// this includes `hidden` because "the content must still be scrollable
/// programmatically […] and the box is therefore still a scroll container". Only
/// `overflow: clip` (and `visible` / absent) is excluded — `clip` "forbids
/// scrolling entirely, through any mechanism, and therefore the box is not a scroll
/// container".
#[must_use]
pub(crate) fn establishes_scroll_container(computed: &ComputedStyles, node: NodeId) -> bool {
    matches!(
        computed
            .get(&node)
            .and_then(|style| style.get(PropertyId::Overflow))
            .map(ComputedValue::value),
        Some(&PropertyValue::Keyword(sym))
            if sym == layout_keyword::OVERFLOW_SCROLL
                || sym == layout_keyword::OVERFLOW_AUTO
                || sym == layout_keyword::OVERFLOW_HIDDEN
    )
}

/// The scrollable content height of `node` (logical px), measured from its
/// scrollport top down to the bottom-most descendant edge (CSS Overflow 3
/// §scrollable-overflow-region), never less than the scrollport height.
///
/// The walk descends the node's subtree but **stops at any nested scroll
/// container** (its own border box already bounds its overflow), so an inner
/// scroller's tall content does not inflate the outer scroller's extent.
#[must_use]
fn scrollable_content_height(
    tree: &RetainedTree,
    computed: &ComputedStyles,
    boxes: &LayoutBoxes,
    node: NodeId,
    scrollport: Rect,
) -> f32 {
    let top = scrollport.origin.y;
    let mut max_bottom = scrollport.bottom();
    let mut stack: Vec<NodeId> = tree
        .get(node)
        .map_or_else(Vec::new, |n| n.child_nodes().to_vec());
    let mut visited: HashSet<NodeId> = HashSet::new();
    while let Some(child) = stack.pop() {
        if !visited.insert(child) {
            continue;
        }
        if let Some(b) = boxes.get(&child) {
            max_bottom = max_bottom.max(b.bounds.bottom());
        }
        // Do not descend into a nested scroll container: it clips its own overflow.
        if !establishes_scroll_container(computed, child)
            && let Some(n) = tree.get(child)
        {
            stack.extend(n.child_nodes().iter().copied());
        }
    }
    max_bottom - top
}

/// Enumerates every scroll container in the tree that **currently overflows**,
/// depth-sorted ascending (outermost first) so a consumer can compose their
/// scroll positions outer-to-inner (CSS Overflow 3 §scroll-container).
///
/// A scroll container whose content fits (content height ≤ scrollport height)
/// owns no scroll position and is omitted — with `overflow: auto` it also shows
/// no scrollbar, matching the spec's "as needed" rule.
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
    let mut visited: HashSet<NodeId> = HashSet::new();
    while let Some((node, depth)) = stack.pop() {
        if !visited.insert(node) {
            continue;
        }
        if establishes_scroll_container(computed, node)
            && let Some(b) = boxes.get(&node)
        {
            let scrollport = b.bounds;
            let content_height = scrollable_content_height(tree, computed, boxes, node, scrollport);
            if content_height > scrollport.size.height {
                out.push(ScrollContainer {
                    node,
                    scrollport,
                    content_height,
                    depth,
                });
            }
        }
        if let Some(n) = tree.get(node) {
            for &child in n.child_nodes() {
                stack.push((child, depth.saturating_add(1)));
            }
        }
    }
    out.sort_by_key(|c| c.depth);
    out
}
