#![doc = include_str!("scroll.doc/_module.md")]

use std::collections::HashSet;

use obscur_geometry::Rect;
use obscur_identity::NodeId;
use obscur_style::computed::ComputedValue;
use obscur_style::property::{PropertyId, PropertyValue, layout_keyword};
use obscur_tree::tree::RetainedTree;

use super::{ComputedStyles, LayoutBoxes};

#[doc = include_str!("scroll.doc/ScrollContainer.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ScrollContainer {
    #[doc = include_str!("scroll.doc/node.md")]
    pub node: NodeId,
    #[doc = include_str!("scroll.doc/scrollport.md")]
    pub scrollport: Rect,
    #[doc = include_str!("scroll.doc/content_height.md")]
    pub content_height: f32,
    #[doc = include_str!("scroll.doc/depth.md")]
    pub depth: u32,
}

impl ScrollContainer {
    #[doc = include_str!("scroll.doc/max_scroll_top.md")]
    #[must_use]
    pub(crate) fn max_scroll_top(self) -> f32 {
        (self.content_height - self.scrollport.size.height).max(0.0)
    }
}

#[doc = include_str!("scroll.doc/overflow_scrolls.md")]
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

#[doc = include_str!("scroll.doc/establishes_scroll_container.md")]
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

#[doc = include_str!("scroll.doc/scrollable_content_height.md")]
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

#[doc = include_str!("scroll.doc/scroll_containers.md")]
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
