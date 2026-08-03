#![doc = include_str!("hit_testing.doc/_module.md")]

use std::collections::HashMap;

use obscur_geometry::{Point, Rect};
use obscur_identity::NodeId;
use obscur_interaction::spatial::SpatialIndex;
use obscur_style::computed::ComputedValue;
use obscur_style::property::{Length, PropertyId, PropertyValue};
use obscur_tree::tree::RetainedTree;

use crate::orchestration::{ComputedStyles, LayoutBoxes};

#[doc = include_str!("hit_testing.doc/HitTesting.md")]
pub(crate) struct HitTesting {
    #[doc = include_str!("hit_testing.doc/index.md")]
    index: SpatialIndex,
}

impl HitTesting {
    #[doc = include_str!("hit_testing.doc/new.md")]
    pub(crate) fn new() -> Self {
        Self {
            index: SpatialIndex::new(),
        }
    }

    #[doc = include_str!("hit_testing.doc/rebuild.md")]
    pub(crate) fn rebuild(&mut self, tree: &RetainedTree, boxes: &LayoutBoxes) {
        self.index.clear();
        for (&node, layout_box) in boxes {
            let z_order = Self::logical_depth(tree, node);
            self.index.insert(node, layout_box.bounds, z_order);
        }
    }

    #[doc = include_str!("hit_testing.doc/resolve.md")]
    pub(crate) fn resolve(
        &self,
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
        mapped: Point,
        screen: Point,
    ) -> Option<NodeId> {
        let preorder = Self::preorder_index(tree);
        let mut best: Option<((i32, i32, usize), NodeId)> = None;
        for hit in self.index.hits_at(mapped) {
            let node = hit.node();
            if !Self::is_hittable(tree, computed, boxes, node, mapped, screen) {
                continue;
            }
            let key = (
                Self::used_z_index(computed, node),
                Self::logical_depth(tree, node),
                preorder.get(&node).copied().unwrap_or(0),
            );
            if best.is_none_or(|(bk, _)| key > bk) {
                best = Some((key, node));
            }
        }
        best.map(|(_, node)| node)
    }

    #[doc = include_str!("hit_testing.doc/logical_depth.md")]
    fn logical_depth(tree: &RetainedTree, node: NodeId) -> i32 {
        let mut depth: i32 = 0;
        let mut current = node;
        // The tree is a DAG in practice; cap the walk at the node count so a
        // pathological cycle terminates deterministically.
        let ancestor_walk_limit = tree.len();
        for _ in 0..ancestor_walk_limit {
            match tree.get(current).and_then(|n| n.logical_parent().parent()) {
                Some(parent) => {
                    depth = depth.saturating_add(1);
                    current = parent;
                }
                None => break,
            }
        }
        depth
    }

    #[doc = include_str!("hit_testing.doc/used_z_index.md")]
    // `z-index` is lowered only from an integral `Number` (`fract() == 0.0`), so
    // the truncation is exact across the representable stacking range.
    #[allow(clippy::cast_possible_truncation)]
    fn used_z_index(computed: &ComputedStyles, node: NodeId) -> i32 {
        match computed
            .get(&node)
            .and_then(|style| style.get(PropertyId::ZIndex))
            .map(ComputedValue::value)
        {
            Some(&PropertyValue::Number(n)) => n as i32,
            _ => 0,
        }
    }

    #[doc = include_str!("hit_testing.doc/preorder_index.md")]
    fn preorder_index(tree: &RetainedTree) -> HashMap<NodeId, usize> {
        let mut out = HashMap::new();
        let Some(root) = tree.root() else {
            return out;
        };
        // Explicit stack DFS, children pushed in reverse so they pop in order. The
        // visited guard makes a malformed (cyclic) child list terminate.
        let mut stack = vec![root];
        let mut next = 0usize;
        while let Some(node) = stack.pop() {
            if out.contains_key(&node) {
                continue;
            }
            out.insert(node, next);
            next += 1;
            if let Some(n) = tree.get(node) {
                for &child in n.child_nodes().iter().rev() {
                    stack.push(child);
                }
            }
        }
        out
    }

    #[doc = include_str!("hit_testing.doc/is_hittable.md")]
    fn is_hittable(
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
        node: NodeId,
        mapped: Point,
        screen: Point,
    ) -> bool {
        let ancestor_walk_limit = tree.len();
        let mut current = node;
        for _ in 0..ancestor_walk_limit {
            let Some(parent) = tree.get(current).and_then(|n| n.logical_parent().parent()) else {
                break;
            };
            if Self::overflow_clips(computed, parent)
                && let Some(b) = boxes.get(&parent)
                && !b.bounds.contains(screen)
            {
                return false;
            }
            current = parent;
        }
        if let Some(b) = boxes.get(&node) {
            let radii = Self::corner_radii(computed, node, b.bounds);
            if !rounded_rect_contains(b.bounds, radii, mapped) {
                return false;
            }
        }
        true
    }

    #[doc = include_str!("hit_testing.doc/overflow_clips.md")]
    fn overflow_clips(computed: &ComputedStyles, node: NodeId) -> bool {
        matches!(
            computed
                .get(&node)
                .and_then(|style| style.get(PropertyId::Overflow))
                .map(ComputedValue::value),
            Some(&PropertyValue::Keyword(sym))
                if sym != obscur_style::property::layout_keyword::OVERFLOW_VISIBLE
        )
    }

    #[doc = include_str!("hit_testing.doc/corner_radii.md")]
    fn corner_radii(computed: &ComputedStyles, node: NodeId, bounds: Rect) -> [(f32, f32); 4] {
        let corner = |id: PropertyId| -> (f32, f32) {
            match computed
                .get(&node)
                .and_then(|style| style.get(id))
                .and_then(ComputedValue::as_length)
            {
                Some(Length::Px(px)) => (px.0, px.0),
                Some(Length::Percent(pct)) => (
                    pct / 100.0 * bounds.size.width,
                    pct / 100.0 * bounds.size.height,
                ),
                _ => (0.0, 0.0),
            }
        };
        [
            corner(PropertyId::BorderTopLeftRadius),
            corner(PropertyId::BorderTopRightRadius),
            corner(PropertyId::BorderBottomRightRadius),
            corner(PropertyId::BorderBottomLeftRadius),
        ]
    }
}

#[doc = include_str!("hit_testing.doc/rounded_rect_contains.md")]
fn rounded_rect_contains(bounds: Rect, radii: [(f32, f32); 4], point: Point) -> bool {
    if !bounds.contains(point) {
        return false;
    }
    let (l, t, r, b) = (
        bounds.origin.x,
        bounds.origin.y,
        bounds.right(),
        bounds.bottom(),
    );
    // Clamp each radius so opposing corners never overlap past the box.
    let clamp_pair = |rx: f32, ry: f32| {
        (
            rx.min(bounds.size.width).max(0.0),
            ry.min(bounds.size.height).max(0.0),
        )
    };
    let [(tlx, tly), (trx, tr_y), (brx, bry), (blx, bly)] = radii;
    let (tlx, tly) = clamp_pair(tlx, tly);
    let (trx, tr_y) = clamp_pair(trx, tr_y);
    let (brx, bry) = clamp_pair(brx, bry);
    let (blx, bly) = clamp_pair(blx, bly);
    let outside_ellipse = |px: f32, py: f32, cx: f32, cy: f32, rx: f32, ry: f32| -> bool {
        if rx <= 0.0 || ry <= 0.0 {
            return false;
        }
        let dx = (px - cx) / rx;
        let dy = (py - cy) / ry;
        dx * dx + dy * dy > 1.0
    };
    if point.x < l + tlx
        && point.y < t + tly
        && outside_ellipse(point.x, point.y, l + tlx, t + tly, tlx, tly)
    {
        return false;
    }
    if point.x > r - trx
        && point.y < t + tr_y
        && outside_ellipse(point.x, point.y, r - trx, t + tr_y, trx, tr_y)
    {
        return false;
    }
    if point.x > r - brx
        && point.y > b - bry
        && outside_ellipse(point.x, point.y, r - brx, b - bry, brx, bry)
    {
        return false;
    }
    if point.x < l + blx
        && point.y > b - bly
        && outside_ellipse(point.x, point.y, l + blx, b - bly, blx, bly)
    {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Size;

    #[test]
    fn a_square_box_contains_every_point_inside_it() {
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0));
        assert!(rounded_rect_contains(
            bounds,
            [(0.0, 0.0); 4],
            Point::new(0.5, 0.5)
        ));
    }

    #[test]
    fn a_rounded_corner_excludes_the_point_it_cuts_away() {
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0));
        let radii = [(4.0, 4.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0)];
        // Inside the rectangle, but outside the top-left corner ellipse.
        assert!(!rounded_rect_contains(bounds, radii, Point::new(0.4, 0.4)));
        // The opposite corner is square, so the mirrored point is still inside.
        assert!(rounded_rect_contains(bounds, radii, Point::new(9.6, 9.6)));
    }

    #[test]
    fn an_empty_index_resolves_nothing() {
        let hits = HitTesting::new();
        let tree = RetainedTree::new();
        assert_eq!(
            hits.resolve(
                &tree,
                &ComputedStyles::new(),
                &LayoutBoxes::new(),
                Point::new(1.0, 1.0),
                Point::new(1.0, 1.0),
            ),
            None
        );
    }
}
