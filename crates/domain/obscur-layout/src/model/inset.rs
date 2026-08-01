#![doc = include_str!("inset.doc/_module.md")]

use obscur_geometry::{Point, Rect, Size};

use crate::model::{BoxSizing, Dimension};

#[doc = include_str!("inset.doc/Inset.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Inset {
    #[doc = include_str!("inset.doc/top.md")]
    pub top: Dimension,
    #[doc = include_str!("inset.doc/right.md")]
    pub right: Dimension,
    #[doc = include_str!("inset.doc/bottom.md")]
    pub bottom: Dimension,
    #[doc = include_str!("inset.doc/left.md")]
    pub left: Dimension,
}

impl Inset {
    #[doc = include_str!("inset.doc/AUTO.md")]
    pub const AUTO: Self = Self {
        top: Dimension::Auto,
        right: Dimension::Auto,
        bottom: Dimension::Auto,
        left: Dimension::Auto,
    };

    #[doc = include_str!("inset.doc/new.md")]
    #[must_use]
    pub const fn new(top: Dimension, right: Dimension, bottom: Dimension, left: Dimension) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    #[doc = include_str!("inset.doc/resolve_absolute_box.md")]
    #[must_use]
    pub fn resolve_absolute_box(
        self,
        cb: Rect,
        width: Dimension,
        height: Dimension,
        natural: Size,
        box_sizing: BoxSizing,
        border_padding: Size,
    ) -> Rect {
        let to_border_box = |size: f32, bp: f32| box_sizing.content_extent(size, bp) + bp;
        let (x, w) = resolve_axis(
            self.left.resolve(cb.size.width),
            self.right.resolve(cb.size.width),
            width
                .resolve(cb.size.width)
                .map(|content| to_border_box(content, border_padding.width)),
            cb.origin.x,
            cb.size.width,
            natural.width,
        );
        let (y, h) = resolve_axis(
            self.top.resolve(cb.size.height),
            self.bottom.resolve(cb.size.height),
            height
                .resolve(cb.size.height)
                .map(|content| to_border_box(content, border_padding.height)),
            cb.origin.y,
            cb.size.height,
            natural.height,
        );
        Rect::new(Point::new(x, y), Size::new(w, h))
    }

    #[doc = include_str!("inset.doc/sticky_offset.md")]
    #[must_use]
    pub fn sticky_offset(
        self,
        static_pos: Point,
        node_size: Size,
        cb: Rect,
        scrollport: Rect,
    ) -> (f32, f32) {
        let used_x = sticky_axis(
            self.left.resolve(cb.size.width),
            self.right.resolve(cb.size.width),
            static_pos.x,
            node_size.width,
            cb.origin.x,
            cb.origin.x + cb.size.width,
            scrollport.origin.x,
            scrollport.origin.x + scrollport.size.width,
        );
        let used_y = sticky_axis(
            self.top.resolve(cb.size.height),
            self.bottom.resolve(cb.size.height),
            static_pos.y,
            node_size.height,
            cb.origin.y,
            cb.origin.y + cb.size.height,
            scrollport.origin.y,
            scrollport.origin.y + scrollport.size.height,
        );
        (used_x - static_pos.x, used_y - static_pos.y)
    }
}

#[doc = include_str!("inset.doc/sticky_axis.md")]
// The two insets, the static position, the node extent, and the CB / scrollport
// edges are the cohesive per-axis inputs of one clamp; they are not a public API.
#[allow(clippy::too_many_arguments)]
fn sticky_axis(
    start_inset: Option<f32>,
    end_inset: Option<f32>,
    static_pos: f32,
    node_extent: f32,
    cb_start: f32,
    cb_end: f32,
    port_start: f32,
    port_end: f32,
) -> f32 {
    if let Some(start) = start_inset {
        // Stick to the scrollport start edge (never above static, never past CB).
        let sticky_lower = port_start + start;
        let sticky_upper = cb_end - node_extent;
        sticky_lower.max(static_pos).min(sticky_upper)
    } else if let Some(end) = end_inset {
        // Stick to the scrollport end edge (never below static, never past CB top).
        let sticky_from_end = port_end - end - node_extent;
        sticky_from_end.min(static_pos).max(cb_start)
    } else {
        // No inset on this axis: it is not sticky.
        static_pos
    }
}

#[doc = include_str!("inset.doc/resolve_axis.md")]
fn resolve_axis(
    start: Option<f32>,
    end: Option<f32>,
    explicit: Option<f32>,
    cb_origin: f32,
    cb_extent: f32,
    natural: f32,
) -> (f32, f32) {
    match (start, end) {
        // Pinned to both edges: insets determine the size, explicit/natural ignored.
        (Some(start), Some(end)) => (cb_origin + start, (cb_extent - start - end).max(0.0)),
        // Anchored to the start edge; size from explicit-or-natural.
        (Some(start), None) => (cb_origin + start, explicit.unwrap_or(natural)),
        // Anchored to the end edge; position derived from the size.
        (None, Some(end)) => {
            let size = explicit.unwrap_or(natural);
            (cb_origin + cb_extent - end - size, size)
        }
        // Neither inset: static-position fallback to the containing-block origin.
        (None, None) => (cb_origin, explicit.unwrap_or(natural)),
    }
}

impl Default for Inset {
    #[doc = include_str!("inset.doc/default.md")]
    fn default() -> Self {
        Self::AUTO
    }
}

#[cfg(test)]
mod tests {
    use super::Inset;
    use crate::model::{BoxSizing, Dimension};
    use obscur_geometry::{Point, Rect, Size};

    #[test]
    fn default_is_all_auto() {
        let inset = Inset::default();
        assert!(inset.top.is_auto());
        assert!(inset.right.is_auto());
        assert!(inset.bottom.is_auto());
        assert!(inset.left.is_auto());
        assert_eq!(inset, Inset::AUTO);
    }

    #[test]
    fn new_stores_each_edge() {
        let inset = Inset::new(
            Dimension::Px(1.0),
            Dimension::Px(2.0),
            Dimension::Px(3.0),
            Dimension::Px(4.0),
        );
        assert_eq!(inset.top, Dimension::Px(1.0));
        assert_eq!(inset.right, Dimension::Px(2.0));
        assert_eq!(inset.bottom, Dimension::Px(3.0));
        assert_eq!(inset.left, Dimension::Px(4.0));
    }

    /// A containing block offset from the origin, to prove positioning is
    /// relative to the CB rather than absolute zero.
    const CB: Rect = Rect::new(Point::new(100.0, 100.0), Size::new(200.0, 200.0));

    #[test]
    fn top_left_anchor_uses_natural_size() {
        // `top: 10; left: 20` with an explicit 40x30 size, anchored to the
        // start edges of a CB at (100, 100).
        let inset = Inset::new(
            Dimension::Px(10.0),
            Dimension::Auto,
            Dimension::Auto,
            Dimension::Px(20.0),
        );
        let rect = inset.resolve_absolute_box(
            CB,
            Dimension::Px(40.0),
            Dimension::Px(30.0),
            Size::new(40.0, 30.0),
            BoxSizing::ContentBox,
            Size::new(0.0, 0.0),
        );
        assert_eq!(
            rect,
            Rect::new(Point::new(120.0, 110.0), Size::new(40.0, 30.0))
        );
    }

    #[test]
    fn right_bottom_anchor_derives_position_from_size() {
        // `right: 0; bottom: 0; 20x20` sits flush against the CB's far corner.
        let inset = Inset::new(
            Dimension::Auto,
            Dimension::Px(0.0),
            Dimension::Px(0.0),
            Dimension::Auto,
        );
        let rect = inset.resolve_absolute_box(
            CB,
            Dimension::Px(20.0),
            Dimension::Px(20.0),
            Size::new(20.0, 20.0),
            BoxSizing::ContentBox,
            Size::new(0.0, 0.0),
        );
        // 100 + 200 - 0 - 20 = 280 on each axis.
        assert_eq!(
            rect,
            Rect::new(Point::new(280.0, 280.0), Size::new(20.0, 20.0))
        );
    }

    #[test]
    fn both_insets_stretch_to_containing_block() {
        // `left: 10; right: 30` pins both edges: width = 200 - 10 - 30 = 160,
        // overriding the (ignored) explicit/natural width.
        let inset = Inset::new(
            Dimension::Px(10.0),
            Dimension::Px(30.0),
            Dimension::Px(10.0),
            Dimension::Px(10.0),
        );
        let rect = inset.resolve_absolute_box(
            CB,
            Dimension::Px(5.0),
            Dimension::Auto,
            Size::new(5.0, 5.0),
            BoxSizing::ContentBox,
            Size::new(0.0, 0.0),
        );
        assert_eq!(rect.origin, Point::new(110.0, 110.0));
        assert_eq!(rect.size, Size::new(160.0, 180.0));
    }

    #[test]
    fn no_insets_falls_back_to_containing_block_origin() {
        // Neither inset on either axis: static-position fallback to cb origin.
        let rect = Inset::AUTO.resolve_absolute_box(
            CB,
            Dimension::Auto,
            Dimension::Auto,
            Size::new(40.0, 30.0),
            BoxSizing::ContentBox,
            Size::new(0.0, 0.0),
        );
        assert_eq!(
            rect,
            Rect::new(Point::new(100.0, 100.0), Size::new(40.0, 30.0))
        );
    }

    #[test]
    fn percent_insets_resolve_against_containing_block() {
        // `left: 25%; top: 10%` of a 200x200 CB → offsets of 50 and 20.
        let inset = Inset::new(
            Dimension::Percent(10.0),
            Dimension::Auto,
            Dimension::Auto,
            Dimension::Percent(25.0),
        );
        let rect = inset.resolve_absolute_box(
            CB,
            Dimension::Px(10.0),
            Dimension::Px(10.0),
            Size::new(10.0, 10.0),
            BoxSizing::ContentBox,
            Size::new(0.0, 0.0),
        );
        assert_eq!(rect.origin, Point::new(150.0, 120.0));
    }

    /// A `top:10px` sticky node whose static top is 50, inside a tall CB, and a
    /// 300-tall scrollport. Its used top follows the clamp as the scroll offset
    /// grows: static → stuck to scrollport → pinned within the CB.
    #[test]
    fn sticky_top_clamps_static_scrollport_then_containing_block() {
        // `top: 10px`, no other insets.
        let inset = Inset::new(
            Dimension::Px(10.0),
            Dimension::Auto,
            Dimension::Auto,
            Dimension::Auto,
        );
        // Node 20 tall; static origin (0, 50); a tall CB (0..1000) so the upper
        // clamp is `1000 - 20 = 980`.
        let node = Size::new(20.0, 20.0);
        let static_pos = Point::new(0.0, 50.0);
        let cb = Rect::new(Point::new(0.0, 0.0), Size::new(300.0, 1000.0));
        let port = Size::new(300.0, 300.0);

        // scroll 0: sticky_lower = 0 + 10 = 10 <= static 50 → stays at static.
        let scrollport = Rect::new(Point::new(0.0, 0.0), port);
        let (dx, dy) = inset.sticky_offset(static_pos, node, cb, scrollport);
        assert_eq!((dx, dy), (0.0, 0.0), "scroll 0: sticky at static");

        // scroll 100: sticky_lower = 100 + 10 = 110 > static 50 → sticks at 110.
        let scrollport = Rect::new(Point::new(0.0, 100.0), port);
        let (dx, dy) = inset.sticky_offset(static_pos, node, cb, scrollport);
        assert_eq!(dx, 0.0, "no horizontal inset → no x offset");
        assert_eq!(dy, 60.0, "used top 110 = static 50 + dy 60");

        // huge scroll: sticky_lower would be 5000 + 10, clamped to cb.bottom - h
        // = 1000 - 20 = 980 → pinned within the CB. dy = 980 - 50 = 930.
        let scrollport = Rect::new(Point::new(0.0, 5000.0), port);
        let (_dx, dy) = inset.sticky_offset(static_pos, node, cb, scrollport);
        assert_eq!(dy, 930.0, "pinned at cb.bottom - height");
    }

    /// A `bottom:10px` sticky node sticks to the scrollport BOTTOM edge, clamped
    /// so it never rises above its static position nor past the CB top.
    #[test]
    fn sticky_bottom_clamps_against_scrollport_bottom() {
        let inset = Inset::new(
            Dimension::Auto,
            Dimension::Auto,
            Dimension::Px(10.0),
            Dimension::Auto,
        );
        let node = Size::new(20.0, 20.0);
        let static_pos = Point::new(0.0, 50.0);
        let cb = Rect::new(Point::new(0.0, 0.0), Size::new(300.0, 1000.0));
        let port = Size::new(300.0, 300.0);

        // scroll 0: scrollport bottom = 300; sticky_from_end = 300 - 10 - 20 = 270.
        // min(270, static 50) = 50 → at static.
        let scrollport = Rect::new(Point::new(0.0, 0.0), port);
        assert_eq!(
            inset.sticky_offset(static_pos, node, cb, scrollport),
            (0.0, 0.0),
            "bottom sticky at static while the sticky line is below it"
        );

        // Scrollport pulled up so its bottom is at 60: sticky_from_end = 60-10-20
        // = 30 < static 50 → sticks upward to used top 30. dy = 30 - 50 = -20.
        let scrollport = Rect::new(Point::new(0.0, -240.0), port);
        let (_dx, dy) = inset.sticky_offset(static_pos, node, cb, scrollport);
        assert_eq!(
            dy, -20.0,
            "bottom sticky pulls the node up to the sticky line"
        );
    }

    /// With no inset on an axis, that axis is not sticky (zero delta).
    #[test]
    fn sticky_with_no_inset_is_inert() {
        let node = Size::new(20.0, 20.0);
        let static_pos = Point::new(30.0, 50.0);
        let cb = Rect::new(Point::new(0.0, 0.0), Size::new(300.0, 1000.0));
        let scrollport = Rect::new(Point::new(0.0, 500.0), Size::new(300.0, 300.0));
        assert_eq!(
            Inset::AUTO.sticky_offset(static_pos, node, cb, scrollport),
            (0.0, 0.0),
            "no insets → no sticking on either axis"
        );
    }
}
