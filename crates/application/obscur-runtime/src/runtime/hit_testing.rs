#![doc = include_str!("hit_testing.doc/_module.md")]

use std::collections::{HashMap, HashSet};

use obscur_geometry::{AffineTransform, Point, Rect};
use obscur_identity::{FastMap, FastSet, NodeId};
use obscur_interaction::spatial::SpatialIndex;
use obscur_scene::path::PathTrim;
use obscur_style::computed::{ComputedValue, OverflowAxes};
use obscur_style::property::{Length, PropertyId, PropertyValue};
use obscur_tree::tree::RetainedTree;

use crate::orchestration::{
    ComputedStyles, LayoutBoxes, PaintLayer, effective_stacking, resolved_transform,
};

#[doc = include_str!("hit_testing.doc/HitTesting.md")]
pub(crate) struct HitTesting {
    #[doc = include_str!("hit_testing.doc/index.md")]
    index: SpatialIndex,
    #[doc = include_str!("hit_testing.doc/canvas_regions.md")]
    canvas_regions: FastMap<NodeId, PathTrim>,
    #[doc = include_str!("hit_testing.doc/inverse_transforms.md")]
    inverse_transforms: FastMap<NodeId, AffineTransform>,
}

impl HitTesting {
    #[doc = include_str!("hit_testing.doc/new.md")]
    pub(crate) fn new() -> Self {
        Self {
            index: SpatialIndex::new(),
            canvas_regions: HashMap::default(),
            inverse_transforms: HashMap::default(),
        }
    }

    #[doc = include_str!("hit_testing.doc/set_canvas_regions.md")]
    pub(crate) fn set_canvas_regions(&mut self, regions: FastMap<NodeId, PathTrim>) {
        self.canvas_regions = regions;
    }

    #[doc = include_str!("hit_testing.doc/rebuild.md")]
    pub(crate) fn rebuild(
        &mut self,
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
    ) {
        self.index.clear();
        self.inverse_transforms.clear();
        let mut memo: FastMap<NodeId, Option<AffineTransform>> = HashMap::default();
        for (&node, layout_box) in boxes {
            let z_order = Self::logical_depth(tree, node);
            let Some(chain) = Self::accumulated_transform(tree, computed, boxes, node, &mut memo)
            else {
                self.index.insert(node, layout_box.bounds, z_order);
                continue;
            };
            // A singular chain (a `scale(0)` anywhere above) collapses the box to
            // zero area. It paints nothing, so it is hit by nothing — leaving it
            // out of the index is both the truth and the cheap answer.
            let Some(inverse) = chain.inverse() else {
                continue;
            };
            self.index
                .insert(node, chain.map_rect(layout_box.bounds), z_order);
            self.inverse_transforms.insert(node, inverse);
        }
    }

    #[doc = include_str!("hit_testing.doc/accumulated_transform.md")]
    fn accumulated_transform(
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
        node: NodeId,
        memo: &mut FastMap<NodeId, Option<AffineTransform>>,
    ) -> Option<AffineTransform> {
        if let Some(&hit) = memo.get(&node) {
            return hit;
        }
        let mut pending = Vec::new();
        let mut inherited = None;
        let mut current = node;
        let ancestor_walk_limit = tree.len();
        for _ in 0..=ancestor_walk_limit {
            if let Some(&known) = memo.get(&current) {
                inherited = known;
                break;
            }
            pending.push(current);
            match tree.get(current).and_then(|n| n.logical_parent().parent()) {
                Some(parent) => current = parent,
                None => break,
            }
        }
        let mut chain = inherited;
        for &ancestor in pending.iter().rev() {
            let own = boxes
                .get(&ancestor)
                .and_then(|b| resolved_transform(computed, ancestor, b.bounds));
            chain = match (chain, own) {
                (Some(outer), Some(inner)) => Some(outer.multiply(inner)),
                (outer, None) => outer,
                (None, inner) => inner,
            };
            memo.insert(ancestor, chain);
        }
        chain
    }

    #[doc = include_str!("hit_testing.doc/local_point.md")]
    fn local_point(&self, node: NodeId, point: Point) -> Point {
        match self.inverse_transforms.get(&node) {
            Some(inverse) => inverse.apply(point),
            None => point,
        }
    }

    #[doc = include_str!("hit_testing.doc/resolve.md")]
    pub(crate) fn resolve(
        &self,
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
        screen: Point,
        scrolls: &FastMap<NodeId, f32>,
    ) -> Option<NodeId> {
        let preorder = Self::preorder_index(tree);
        let stacking = effective_stacking(tree, computed);
        let (zmap, layermap) = (stacking.z, stacking.layer);
        let mut best: Option<((i32, PaintLayer, i32, usize), NodeId)> = None;
        let mut seen: FastSet<NodeId> = HashSet::default();
        for probe in Self::query_points(tree, screen, scrolls) {
            for hit in self.index.hits_at(probe) {
                let node = hit.node();
                // A node surfaced by one probe is judged once, at ITS point: the
                // probe that found it is the broad phase's business, not the
                // answer's.
                if !seen.insert(node) {
                    continue;
                }
                if !self.is_hittable(tree, computed, boxes, scrolls, node, screen) {
                    continue;
                }
                // The canvas region, like the border box, is expressed in the node's
                // own untransformed space, so a rotated canvas needs the point
                // brought back to it too.
                let local = self.local_point(node, Self::folded_point(tree, scrolls, node, screen));
                // §4.13's Canvas custom hit test. A canvas that declared a shape is
                // hit only inside it, so the corners of a round knob's box belong to
                // whatever is behind it — which is the whole reason the hook exists.
                // A canvas that declared none is hit anywhere in its box, like every
                // other element.
                if let Some(region) = self.canvas_regions.get(&node)
                    && !region.contains(local)
                {
                    continue;
                }
                let key = (
                    zmap.get(&node).copied().unwrap_or(0),
                    layermap
                        .get(&node)
                        .copied()
                        .unwrap_or(PaintLayer::NonPositioned),
                    Self::logical_depth(tree, node),
                    preorder.get(&node).copied().unwrap_or(0),
                );
                if best.is_none_or(|(bk, _)| key > bk) {
                    best = Some((key, node));
                }
            }
        }
        best.map(|(_, node)| node)
    }

    #[doc = include_str!("hit_testing.doc/folded_point.md")]
    pub(crate) fn folded_point(
        tree: &RetainedTree,
        scrolls: &FastMap<NodeId, f32>,
        node: NodeId,
        screen: Point,
    ) -> Point {
        Point::new(screen.x, screen.y + Self::scroll_fold(tree, scrolls, node))
    }

    #[doc = include_str!("hit_testing.doc/content_point.md")]
    pub(crate) fn content_point(
        tree: &RetainedTree,
        scrolls: &FastMap<NodeId, f32>,
        node: NodeId,
        screen: Point,
    ) -> Point {
        let own = scrolls.get(&node).copied().unwrap_or(0.0);
        let folded = Self::folded_point(tree, scrolls, node, screen);
        Point::new(folded.x, folded.y + own)
    }

    #[doc = include_str!("hit_testing.doc/scroll_fold.md")]
    fn scroll_fold(tree: &RetainedTree, scrolls: &FastMap<NodeId, f32>, node: NodeId) -> f32 {
        if scrolls.is_empty() {
            return 0.0;
        }
        let mut total = 0.0;
        let mut current = node;
        let ancestor_walk_limit = tree.len();
        for _ in 0..ancestor_walk_limit {
            let Some(parent) = tree.get(current).and_then(|n| n.logical_parent().parent()) else {
                break;
            };
            total += scrolls.get(&parent).copied().unwrap_or(0.0);
            current = parent;
        }
        total
    }

    #[doc = include_str!("hit_testing.doc/query_points.md")]
    fn query_points(
        tree: &RetainedTree,
        screen: Point,
        scrolls: &FastMap<NodeId, f32>,
    ) -> Vec<Point> {
        let mut points = vec![screen];
        for (&container, &own) in scrolls {
            let fold = Self::scroll_fold(tree, scrolls, container) + own;
            // A container scrolled nowhere (and under nothing scrolled) adds no
            // probe: its subtree is already covered by the screen point.
            if fold.abs() > 0.0 {
                points.push(Point::new(screen.x, screen.y + fold));
            }
        }
        points
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

    #[doc = include_str!("hit_testing.doc/preorder_index.md")]
    fn preorder_index(tree: &RetainedTree) -> FastMap<NodeId, usize> {
        let mut out = HashMap::default();
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
        &self,
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
        scrolls: &FastMap<NodeId, f32>,
        node: NodeId,
        screen: Point,
    ) -> bool {
        if Self::is_pointer_transparent(computed, node) {
            return false;
        }
        let ancestor_walk_limit = tree.len();
        let mut current = node;
        // Walking up, the fold sheds one term per step: a node's fold is its
        // parent's fold plus whatever that parent scrolls, so subtracting as we
        // climb costs one lookup where recomputing the chain costs a walk.
        let mut fold = Self::scroll_fold(tree, scrolls, node);
        for _ in 0..ancestor_walk_limit {
            let Some(parent) = tree.get(current).and_then(|n| n.logical_parent().parent()) else {
                break;
            };
            fold -= scrolls.get(&parent).copied().unwrap_or(0.0);
            // The clip is that ancestor's own border box, which a transform on or
            // above it moves on screen; the point has to enter the ancestor's
            // space before it can be compared against the untransformed rect —
            // and that space is scrolled by whatever scrolls ABOVE the ancestor,
            // not by the ancestor's own scroll, which never moves its own box.
            if Self::overflow_clips(computed, parent)
                && let Some(b) = boxes.get(&parent)
                && !b
                    .bounds
                    .contains(self.local_point(parent, Point::new(screen.x, screen.y + fold)))
            {
                return false;
            }
            current = parent;
        }
        if let Some(b) = boxes.get(&node) {
            let radii = Self::corner_radii(computed, node, b.bounds);
            let at = self.local_point(node, Self::folded_point(tree, scrolls, node, screen));
            if !rounded_rect_contains(b.bounds, radii, at) {
                return false;
            }
        }
        true
    }

    #[doc = include_str!("hit_testing.doc/is_pointer_transparent.md")]
    fn is_pointer_transparent(computed: &ComputedStyles, node: NodeId) -> bool {
        matches!(
            computed
                .get(&node)
                .and_then(|style| style.get(PropertyId::PointerEvents))
                .map(ComputedValue::value),
            Some(&PropertyValue::Keyword(sym))
                if sym == obscur_style::property::layout_keyword::POINTER_EVENTS_NONE
        )
    }

    #[doc = include_str!("hit_testing.doc/overflow_clips.md")]
    fn overflow_clips(computed: &ComputedStyles, node: NodeId) -> bool {
        OverflowAxes::of(computed.get(&node)).clips()
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
    use obscur_identity::ComponentId;
    use obscur_layout::model::LayoutBox;
    use obscur_style::computed::ComputedStyle;
    use obscur_style::property::Transform;
    use obscur_style::property::layout_keyword;
    use obscur_style::property::layout_keyword::{POINTER_EVENTS_AUTO, POINTER_EVENTS_NONE};
    use obscur_tree::node::{LogicalParent, NodeKind};

    /// `root → [button, overlay]`, both children filling the same 100x100 box.
    /// The overlay is second in document order, so with no pointer-event policy
    /// it wins the hit and swallows the click.
    fn button_under_overlay() -> (RetainedTree, LayoutBoxes, HitTesting, NodeId, NodeId) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let mut child = || {
            let id = tree.alloc(NodeKind::Element {
                component: ComponentId::from_raw(1),
            });
            tree.get_mut(id)
                .expect("just allocated")
                .set_logical_parent(LogicalParent::of(root));
            tree.get_mut(root).expect("in tree").push_child(id);
            id
        };
        let button = child();
        let overlay = child();

        let mut boxes = LayoutBoxes::default();
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 100.0));
        boxes.insert(button, LayoutBox::new(bounds));
        boxes.insert(overlay, LayoutBox::new(bounds));

        let mut hits = HitTesting::new();
        hits.rebuild(&tree, &ComputedStyles::default(), &boxes);
        (tree, boxes, hits, button, overlay)
    }

    /// The scroll map of a frame in which nothing scrolls — the shape every
    /// pre-scroll-container test ran in, and the fast path of the folding.
    fn unscrolled() -> FastMap<NodeId, f32> {
        HashMap::default()
    }

    /// Allocates an element under `parent` and links it both ways.
    fn kid(tree: &mut RetainedTree, parent: NodeId) -> NodeId {
        let id = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(1),
        });
        tree.get_mut(id)
            .expect("just allocated")
            .set_logical_parent(LogicalParent::of(parent));
        tree.get_mut(parent).expect("in tree").push_child(id);
        id
    }

    fn styled(node: NodeId, keyword: obscur_identity::InternedSymbol) -> ComputedStyles {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::PointerEvents, ComputedValue::keyword(keyword));
        let mut computed = ComputedStyles::default();
        computed.insert(node, style);
        computed
    }

    #[test]
    fn an_overlay_without_the_declaration_swallows_the_click() {
        // The control for the test below: nothing about the geometry lets the
        // button win, so a pass there is the property doing the work.
        let (tree, boxes, hits, _button, overlay) = button_under_overlay();
        let at = Point::new(50.0, 50.0);
        assert_eq!(
            hits.resolve(&tree, &ComputedStyles::default(), &boxes, at, &unscrolled()),
            Some(overlay)
        );
    }

    #[test]
    fn a_pointer_events_none_overlay_lets_the_click_reach_the_button() {
        let (tree, boxes, hits, button, overlay) = button_under_overlay();
        let computed = styled(overlay, POINTER_EVENTS_NONE);
        let at = Point::new(50.0, 50.0);
        assert_eq!(
            hits.resolve(&tree, &computed, &boxes, at, &unscrolled()),
            Some(button),
            "the overlay is transparent to the pointer (§4.7)"
        );
        assert!(!hits.is_hittable(&tree, &computed, &boxes, &unscrolled(), overlay, at));
        assert!(hits.is_hittable(&tree, &computed, &boxes, &unscrolled(), button, at));
    }

    /// `root → [scroller → item, overlay]`, all three filling the same box, the
    /// overlay raised with `z-index: 1` so it outranks the deeper item. This is
    /// the shape the §4.7 spec test drives through real CSS; reproducing the
    /// ranking here keeps the claim it rests on — that the overlay would win —
    /// checkable at a layer that can be run in isolation.
    fn item_under_raised_overlay() -> (RetainedTree, LayoutBoxes, HitTesting, NodeId, NodeId) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let element = |parent: NodeId, tree: &mut RetainedTree| {
            let id = tree.alloc(NodeKind::Element {
                component: ComponentId::from_raw(1),
            });
            tree.get_mut(id)
                .expect("just allocated")
                .set_logical_parent(LogicalParent::of(parent));
            tree.get_mut(parent).expect("in tree").push_child(id);
            id
        };
        let scroller = element(root, &mut tree);
        let item = element(scroller, &mut tree);
        let overlay = element(root, &mut tree);

        let mut boxes = LayoutBoxes::default();
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(200.0, 200.0));
        for node in [scroller, item, overlay] {
            boxes.insert(node, LayoutBox::new(bounds));
        }
        let mut hits = HitTesting::new();
        hits.rebuild(&tree, &ComputedStyles::default(), &boxes);
        (tree, boxes, hits, item, overlay)
    }

    #[test]
    fn a_raised_overlay_outranks_a_deeper_node_until_it_is_transparent() {
        let (tree, boxes, hits, item, overlay) = item_under_raised_overlay();
        let at = Point::new(50.0, 50.0);

        let mut raised = ComputedStyle::new();
        // Positioned, because `z-index` applies to nothing else (CSS 2.1 §9.9.1)
        // and the hit test now says so.
        raised.set(
            PropertyId::Position,
            ComputedValue::keyword(layout_keyword::POSITION_ABSOLUTE),
        );
        raised.set(PropertyId::ZIndex, ComputedValue::number(1.0));
        let mut computed = ComputedStyles::default();
        computed.insert(overlay, raised.clone());
        assert_eq!(
            hits.resolve(&tree, &computed, &boxes, at, &unscrolled()),
            Some(overlay),
            "z-index outranks the deeper item, so the overlay swallows the click"
        );

        raised.set(
            PropertyId::PointerEvents,
            ComputedValue::keyword(POINTER_EVENTS_NONE),
        );
        computed.insert(overlay, raised);
        assert_eq!(
            hits.resolve(&tree, &computed, &boxes, at, &unscrolled()),
            Some(item),
            "made transparent, the same overlay stops swallowing it"
        );
    }

    #[test]
    fn an_explicit_auto_is_hittable_like_an_undeclared_one() {
        // `pointer-events` inherits, so a descendant of a `none` subtree that
        // declares `auto` computes `auto` and must stay hittable — the check
        // reads the node's own computed value for exactly that reason.
        let (tree, boxes, hits, _button, overlay) = button_under_overlay();
        let computed = styled(overlay, POINTER_EVENTS_AUTO);
        let at = Point::new(50.0, 50.0);
        assert_eq!(
            hits.resolve(&tree, &computed, &boxes, at, &unscrolled()),
            Some(overlay)
        );
    }

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

    /// `root → card`, the card arranged at `bounds` and optionally carrying a
    /// CSS `transform` with the default `transform-origin` (the box centre).
    fn card_with_transform(
        matrix: Option<[f32; 6]>,
        bounds: Rect,
    ) -> (
        RetainedTree,
        LayoutBoxes,
        ComputedStyles,
        HitTesting,
        NodeId,
    ) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let card = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(1),
        });
        tree.get_mut(card)
            .expect("just allocated")
            .set_logical_parent(LogicalParent::of(root));
        tree.get_mut(root).expect("in tree").push_child(card);

        let mut boxes = LayoutBoxes::default();
        boxes.insert(card, LayoutBox::new(bounds));

        let mut computed = ComputedStyles::default();
        if let Some(matrix) = matrix {
            let mut style = ComputedStyle::new();
            style.set(
                PropertyId::Transform,
                ComputedValue::from_concrete(PropertyValue::Transform(Transform::new(matrix)))
                    .expect("a matrix transform is a concrete value"),
            );
            computed.insert(card, style);
        }
        let mut hits = HitTesting::new();
        hits.rebuild(&tree, &computed, &boxes);
        (tree, boxes, computed, hits, card)
    }

    fn rotation(radians: f32) -> [f32; 6] {
        let (sin, cos) = radians.sin_cos();
        [cos, sin, -sin, cos, 0.0, 0.0]
    }

    const CARD: Rect = Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 40.0));

    #[test]
    fn a_rotated_card_is_dead_in_the_corners_of_its_bounding_box() {
        // A 45° rotation about the box centre (50, 20). (2, 2) is inside the
        // *layout* rect — so the pre-fix code, which tested the raw screen point
        // against the untransformed box, reported a hit — but inverse-rotating it
        // lands at local (3.3, 41.2), which is past the box's 40px bottom edge.
        // Nothing is drawn there, so nothing may be clicked there.
        let (tree, boxes, computed, hits, _card) =
            card_with_transform(Some(rotation(std::f32::consts::FRAC_PI_4)), CARD);
        let at = Point::new(2.0, 2.0);
        assert_eq!(
            hits.resolve(&tree, &computed, &boxes, at, &unscrolled()),
            None,
            "the rotated card does not cover its bounding box's top-left corner"
        );
    }

    #[test]
    fn a_rotated_card_is_alive_where_it_was_actually_drawn() {
        // The other half of the same rotation: local (95, 5) — a real point of the
        // card's body, near its top-right corner — is painted at (92.4, 41.2),
        // BELOW the untransformed box, where the pre-fix code found nothing at
        // all. The index's broad phase has to keep that point, which is why it
        // holds the bounding box of the transformed corners.
        let (tree, boxes, computed, hits, card) =
            card_with_transform(Some(rotation(std::f32::consts::FRAC_PI_4)), CARD);
        let at = Point::new(92.43, 41.21);
        assert_eq!(
            hits.resolve(&tree, &computed, &boxes, at, &unscrolled()),
            Some(card)
        );
        // The centre is the rotation's fixed point and is hit either way.
        let centre = Point::new(50.0, 20.0);
        assert_eq!(
            hits.resolve(&tree, &computed, &boxes, centre, &unscrolled()),
            Some(card)
        );
    }

    #[test]
    fn a_scaled_card_is_hit_over_the_area_it_grew_into() {
        // `scale(2)` about the centre stretches the card to x ∈ [-50, 150].
        let (tree, boxes, computed, hits, card) =
            card_with_transform(Some([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]), CARD);
        let at = Point::new(120.0, 20.0);
        assert_eq!(
            hits.resolve(&tree, &computed, &boxes, at, &unscrolled()),
            Some(card),
            "local (85, 20) is inside the card, so the pixel it was drawn on is live"
        );
    }

    #[test]
    fn a_shrunken_card_is_not_hit_where_it_pulled_away_from() {
        // `scale(0.5)` about the centre leaves the card covering x ∈ [25, 75];
        // (90, 20) is bare background that the layout rect still claims.
        let (tree, boxes, computed, hits, card) =
            card_with_transform(Some([0.5, 0.0, 0.0, 0.5, 0.0, 0.0]), CARD);
        let at = Point::new(90.0, 20.0);
        assert_eq!(
            hits.resolve(&tree, &computed, &boxes, at, &unscrolled()),
            None
        );
        let inside = Point::new(70.0, 20.0);
        assert_eq!(
            hits.resolve(&tree, &computed, &boxes, inside, &unscrolled()),
            Some(card)
        );
    }

    #[test]
    fn a_translated_card_moves_its_hit_region_with_it() {
        let (tree, boxes, computed, hits, card) =
            card_with_transform(Some([1.0, 0.0, 0.0, 1.0, 30.0, 0.0]), CARD);
        let moved_onto = Point::new(110.0, 20.0);
        assert_eq!(
            hits.resolve(&tree, &computed, &boxes, moved_onto, &unscrolled()),
            Some(card)
        );
        let vacated = Point::new(10.0, 20.0);
        assert_eq!(
            hits.resolve(&tree, &computed, &boxes, vacated, &unscrolled()),
            None
        );
    }

    #[test]
    fn a_card_arranged_away_from_the_origin_is_hit_at_its_box() {
        // No transform anywhere: the plain layout offset the pre-fix code already
        // handled correctly, and which must stay correct.
        let offset = Rect::new(Point::new(30.0, 12.0), Size::new(100.0, 40.0));
        let (tree, boxes, computed, hits, card) = card_with_transform(None, offset);
        let inside = Point::new(35.0, 15.0);
        assert_eq!(
            hits.resolve(&tree, &computed, &boxes, inside, &unscrolled()),
            Some(card)
        );
        let outside = Point::new(20.0, 15.0);
        assert_eq!(
            hits.resolve(&tree, &computed, &boxes, outside, &unscrolled()),
            None
        );
    }

    #[test]
    fn an_untransformed_node_takes_the_shortcut_and_is_indexed_verbatim() {
        // The common case pays for nothing: no accumulated transform is stored,
        // so the pointer path does one failed lookup and no matrix arithmetic,
        // and the index holds the layout box itself rather than a mapped copy.
        let (_tree, _boxes, _computed, hits, card) = card_with_transform(None, CARD);
        assert!(
            hits.inverse_transforms.is_empty(),
            "an untransformed tree stores no inverses"
        );
        let entry = hits
            .index
            .topmost_at(Point::new(1.0, 1.0))
            .expect("the card is indexed");
        assert_eq!(entry.node(), card);
    }

    #[test]
    fn a_collapsed_card_is_hit_by_nothing() {
        // `scale(0, 1)` is singular: the card paints to a line of zero area, so
        // there is no inverse and no point of the plane belongs to it.
        let (tree, boxes, computed, hits, _card) =
            card_with_transform(Some([0.0, 0.0, 0.0, 1.0, 0.0, 0.0]), CARD);
        for at in [
            Point::new(50.0, 20.0),
            Point::new(2.0, 2.0),
            Point::new(99.0, 39.0),
        ] {
            assert_eq!(
                hits.resolve(&tree, &computed, &boxes, at, &unscrolled()),
                None
            );
        }
    }

    #[test]
    fn an_empty_index_resolves_nothing() {
        let hits = HitTesting::new();
        let tree = RetainedTree::new();
        assert_eq!(
            hits.resolve(
                &tree,
                &ComputedStyles::default(),
                &LayoutBoxes::default(),
                Point::new(1.0, 1.0),
                &unscrolled(),
            ),
            None
        );
    }

    // ---- paint order: `z-index` is per stacking context (§9.9) --------------

    fn positioned(z: Option<f32>) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::Position,
            ComputedValue::keyword(layout_keyword::POSITION_ABSOLUTE),
        );
        if let Some(z) = z {
            style.set(PropertyId::ZIndex, ComputedValue::number(z));
        }
        style
    }

    const WIDE: Size = Size::new(200.0, 200.0);

    #[test]
    fn a_raised_node_in_a_low_context_loses_to_a_plain_node_in_a_high_one() {
        // `root → [early(z:0) → deep(z:5), late(z:10) → plain]`. `late`'s own box
        // is a corner the probe misses, so the only two candidates are `deep` and
        // `plain` — the pair the defect gets backwards. `plain` declares no
        // `z-index` at all, and paints at 10 because its stacking context does.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let early = kid(&mut tree, root);
        let deep = kid(&mut tree, early);
        let late = kid(&mut tree, root);
        let plain = kid(&mut tree, late);

        let mut boxes = LayoutBoxes::default();
        let full = Rect::new(Point::new(0.0, 0.0), WIDE);
        boxes.insert(early, LayoutBox::new(full));
        boxes.insert(deep, LayoutBox::new(full));
        boxes.insert(
            late,
            LayoutBox::new(Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0))),
        );
        boxes.insert(plain, LayoutBox::new(full));

        let mut computed = ComputedStyles::default();
        computed.insert(early, positioned(Some(0.0)));
        computed.insert(deep, positioned(Some(5.0)));
        computed.insert(late, positioned(Some(10.0)));
        computed.insert(plain, ComputedStyle::new());

        let mut hits = HitTesting::new();
        hits.rebuild(&tree, &computed, &boxes);

        let at = Point::new(50.0, 50.0);
        let winner = hits.resolve(&tree, &computed, &boxes, at, &unscrolled());
        assert_ne!(
            winner,
            Some(deep),
            "`z-index: 5` inside a `z-index: 0` context must not outrank a \
             `z-index: 10` context's content — that is comparing z globally"
        );
        assert_eq!(
            winner,
            Some(plain),
            "the click lands on what paints on top: the z:10 subtree"
        );
    }

    #[test]
    fn a_z_index_on_a_static_box_is_ignored_until_it_is_positioned() {
        // `root → [a(z:9), b]`, same box. `z-index` applies to positioned
        // elements only (CSS 2.1 §9.9.1), so while `a` is static the two are
        // equal and document order gives the click to `b`.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = kid(&mut tree, root);
        let b = kid(&mut tree, root);

        let mut boxes = LayoutBoxes::default();
        let full = Rect::new(Point::new(0.0, 0.0), WIDE);
        boxes.insert(a, LayoutBox::new(full));
        boxes.insert(b, LayoutBox::new(full));

        let mut raised_static = ComputedStyle::new();
        raised_static.set(PropertyId::ZIndex, ComputedValue::number(9.0));
        let mut computed = ComputedStyles::default();
        computed.insert(a, raised_static);
        computed.insert(b, ComputedStyle::new());

        let mut hits = HitTesting::new();
        hits.rebuild(&tree, &computed, &boxes);
        let at = Point::new(50.0, 50.0);
        assert_eq!(
            hits.resolve(&tree, &computed, &boxes, at, &unscrolled()),
            Some(b),
            "a `z-index` on a `position: static` box lifts nothing"
        );

        // The control: the identical declaration on a POSITIONED `a` does lift
        // it, so the assertion above is the `position` check doing the work and
        // not a hit test that lost track of `z-index` altogether.
        computed.insert(a, positioned(Some(9.0)));
        hits.rebuild(&tree, &computed, &boxes);
        assert_eq!(
            hits.resolve(&tree, &computed, &boxes, at, &unscrolled()),
            Some(a),
        );
    }

    // ---- per-node scroll folding -------------------------------------------

    fn with_keyword(id: PropertyId, keyword: obscur_identity::InternedSymbol) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(id, ComputedValue::keyword(keyword));
        style
    }

    #[test]
    fn a_fixed_toolbar_over_a_scroller_is_clickable_at_every_offset() {
        // `root → [scroller(overflow: scroll) → item, toolbar(position: fixed)]`,
        // the toolbar a 50px strip across the top of the scrolled region. It does
        // not ride the scroll, so its box is where the pointer already is —
        // folding the scroll into the point before matching it against every node
        // moved the probe off it and made it unclickable.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let scroller = kid(&mut tree, root);
        let item = kid(&mut tree, scroller);
        let toolbar = kid(&mut tree, root);

        let mut boxes = LayoutBoxes::default();
        boxes.insert(
            scroller,
            LayoutBox::new(Rect::new(Point::new(0.0, 0.0), Size::new(200.0, 400.0))),
        );
        boxes.insert(
            item,
            LayoutBox::new(Rect::new(Point::new(0.0, 0.0), Size::new(200.0, 2000.0))),
        );
        boxes.insert(
            toolbar,
            LayoutBox::new(Rect::new(Point::new(0.0, 0.0), Size::new(200.0, 50.0))),
        );

        let mut computed = ComputedStyles::default();
        computed.insert(
            scroller,
            with_keyword(PropertyId::OverflowY, layout_keyword::OVERFLOW_SCROLL),
        );
        computed.insert(item, ComputedStyle::new());
        computed.insert(
            toolbar,
            with_keyword(PropertyId::Position, layout_keyword::POSITION_FIXED),
        );

        let mut hits = HitTesting::new();
        hits.rebuild(&tree, &computed, &boxes);

        let on_toolbar = Point::new(50.0, 20.0);
        for offset in [0.0_f32, 100.0, 300.0, 1500.0] {
            let scrolls = FastMap::from_iter([(scroller, offset)]);
            assert_eq!(
                hits.resolve(&tree, &computed, &boxes, on_toolbar, &scrolls),
                Some(toolbar),
                "the toolbar sits still on screen at scroll offset {offset}, so \
                 it must take the click there"
            );
            // And the scrolled content below it still answers at its own folded
            // point, which is the half a global fold got right.
            let below = Point::new(50.0, 300.0);
            assert_eq!(
                hits.resolve(&tree, &computed, &boxes, below, &scrolls),
                Some(item),
                "content inside the scroller is still hit through the fold"
            );
        }
    }

    #[test]
    fn a_clip_two_levels_up_is_tested_in_its_own_scrolled_space() {
        // `root → outer(overflow: scroll, 200x200) → mid(overflow: hidden, y
        // 100..250) → inner(y 200..300)`, with `outer` scrolled 100. `mid` states
        // its box in `outer`'s UNSCROLLED space, so its clip has to be tested at
        // the folded point; testing it at the raw screen point — right only for
        // the first clipping ancestor — leaves `inner` clickable across a band
        // `mid` clipped away.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let outer = kid(&mut tree, root);
        let mid = kid(&mut tree, outer);
        let inner = kid(&mut tree, mid);

        let mut boxes = LayoutBoxes::default();
        boxes.insert(outer, LayoutBox::new(Rect::new(Point::new(0.0, 0.0), WIDE)));
        boxes.insert(
            mid,
            LayoutBox::new(Rect::new(Point::new(0.0, 100.0), Size::new(200.0, 150.0))),
        );
        boxes.insert(
            inner,
            LayoutBox::new(Rect::new(Point::new(0.0, 200.0), Size::new(200.0, 100.0))),
        );

        let mut computed = ComputedStyles::default();
        computed.insert(
            outer,
            with_keyword(PropertyId::OverflowY, layout_keyword::OVERFLOW_SCROLL),
        );
        computed.insert(
            mid,
            with_keyword(PropertyId::OverflowY, layout_keyword::OVERFLOW_HIDDEN),
        );
        computed.insert(inner, ComputedStyle::new());

        let mut hits = HitTesting::new();
        hits.rebuild(&tree, &computed, &boxes);
        let scrolls = FastMap::from_iter([(outer, 100.0_f32)]);

        // `mid` shows content from unscrolled y 100..250, which after the 100px
        // scroll is screen y 0..150. At screen y 170 `inner` is folded to y 270 —
        // inside its own box, and 20px past the bottom of the clip.
        assert_eq!(
            hits.resolve(&tree, &computed, &boxes, Point::new(50.0, 170.0), &scrolls),
            Some(outer),
            "`inner` is clipped away by `mid` there; the click belongs to the \
             scroller behind it"
        );
        // The other half: where `mid` does show `inner`, it is hit — so the
        // assertion above is the clip and not a fold that lost the node.
        assert_eq!(
            hits.resolve(&tree, &computed, &boxes, Point::new(50.0, 120.0), &scrolls),
            Some(inner),
        );
    }
}
