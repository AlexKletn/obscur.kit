//! The four positioning insets of a node (`top` / `right` / `bottom` / `left`).

use obscur_geometry::{Point, Rect, Size};

use crate::model::{BoxSizing, Dimension, Direction, LayoutStyle, Margin, WritingMode};

#[doc = include_str!("../../docs/api/model/inset.doc/Inset.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Inset {
    /// The `top` inset.
    pub top: Dimension,
    /// The `right` inset.
    pub right: Dimension,
    /// The `bottom` inset.
    pub bottom: Dimension,
    /// The `left` inset.
    pub left: Dimension,
}

impl Inset {
    /// The all-`auto` insets — the CSS initial value, contributing no offset.
    pub const AUTO: Self = Self {
        top: Dimension::Auto,
        right: Dimension::Auto,
        bottom: Dimension::Auto,
        left: Dimension::Auto,
    };

    /// Builds insets from the four edges (top / right / bottom / left).
    #[must_use]
    pub const fn new(top: Dimension, right: Dimension, bottom: Dimension, left: Dimension) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Solves positioned geometry with zero margins and the default writing mode. Automatic sizes stretch between paired insets. Definite sizes take precedence.
    #[must_use]
    pub fn resolve_absolute_box(
        &self,
        cb: Rect,
        width: Dimension,
        height: Dimension,
        natural: Size,
        box_sizing: BoxSizing,
        border_padding: Size,
    ) -> Rect {
        let style = LayoutStyle {
            width,
            height,
            box_sizing,
            ..LayoutStyle::default()
        };
        self.resolve_absolute_style(
            cb,
            &style,
            natural,
            border_padding,
            (WritingMode::HorizontalTb, Direction::Ltr),
        )
    }

    /// Solves positioned border-box geometry with margins, sizing constraints and the containing block’s axes.
    #[must_use]
    pub fn resolve_absolute_style(
        &self,
        cb: Rect,
        style: &LayoutStyle,
        natural: Size,
        border_padding: Size,
        containing_axes: (WritingMode, Direction),
    ) -> Rect {
        let (writing, direction) = containing_axes;
        let vertical = writing.is_vertical();
        let inline = if vertical {
            cb.size.height
        } else {
            cb.size.width
        };
        let solve = |extent,
                     basis,
                     insets: (&Dimension, &Dimension),
                     size: (&Dimension, &Dimension, &Dimension),
                     natural,
                     bp,
                     margins: (&Margin, &Margin),
                     reversed,
                     inline_axis| {
            let border_box = |length| style.box_sizing.content_extent(length, bp) + bp;
            resolve_axis(
                extent,
                (
                    insets.0.resolve_signed(extent),
                    insets.1.resolve_signed(extent),
                ),
                size.0.resolve(extent).map_or(natural, border_box),
                size.0.is_auto(),
                size.1.resolve(extent).map_or(bp, border_box),
                size.2.resolve(extent).map_or(f32::INFINITY, border_box),
                (margins.0.resolve(basis), margins.1.resolve(basis)),
                (margins.0.is_auto(), margins.1.is_auto()),
                reversed,
                inline_axis,
            )
        };
        let (x, w) = solve(
            cb.size.width,
            inline,
            (&self.left, &self.right),
            (&style.width, &style.min_width, &style.max_width),
            natural.width,
            border_padding.width,
            (&style.margin.left, &style.margin.right),
            if vertical {
                writing == WritingMode::VerticalRl
            } else {
                direction == Direction::Rtl
            },
            !vertical,
        );
        let (y, h) = solve(
            cb.size.height,
            inline,
            (&self.top, &self.bottom),
            (&style.height, &style.min_height, &style.max_height),
            natural.height,
            border_padding.height,
            (&style.margin.top, &style.margin.bottom),
            vertical && direction == Direction::Rtl,
            vertical,
        );
        Rect::new(
            Point::new(cb.origin.x + x, cb.origin.y + y),
            Size::new(w, h),
        )
    }

    #[doc = include_str!("../../docs/api/model/inset.doc/sticky_offset.md")]
    #[must_use]
    pub fn sticky_offset(
        &self,
        static_pos: Point,
        node_size: Size,
        cb: Rect,
        scrollport: Rect,
    ) -> (f32, f32) {
        let used_x = sticky_axis(
            self.left.resolve_signed(cb.size.width),
            self.right.resolve_signed(cb.size.width),
            static_pos.x,
            node_size.width,
            cb.origin.x,
            cb.origin.x + cb.size.width,
            scrollport.origin.x,
            scrollport.origin.x + scrollport.size.width,
        );
        let used_y = sticky_axis(
            self.top.resolve_signed(cb.size.height),
            self.bottom.resolve_signed(cb.size.height),
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

#[doc = include_str!("../../docs/api/model/inset.doc/sticky_axis.md")]
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
        let sticky_lower = port_start + start;
        let sticky_upper = cb_end - node_extent;
        sticky_lower.max(static_pos).min(sticky_upper)
    } else if let Some(end) = end_inset {
        let sticky_from_end = port_end - end - node_extent;
        sticky_from_end.min(static_pos).max(cb_start)
    } else {
        static_pos
    }
}

/// Solves one positioned axis with insets, margins, and size constraints, returning its physical offset and size.
#[allow(clippy::too_many_arguments)]
fn resolve_axis(
    extent: f32,
    (mut start, mut end): (Option<f32>, Option<f32>),
    preferred: f32,
    auto_size: bool,
    min: f32,
    max: f32,
    (mut lead, mut trail): (f32, f32),
    (mut auto_lead, mut auto_trail): (bool, bool),
    reversed: bool,
    inline: bool,
) -> (f32, f32) {
    if reversed {
        std::mem::swap(&mut start, &mut end);
        std::mem::swap(&mut lead, &mut trail);
        std::mem::swap(&mut auto_lead, &mut auto_trail);
    }
    let clamp = |size: f32| size.min(max).max(min);
    let (offset, size) = match (start, end) {
        (Some(start), Some(end)) => {
            let available = extent - start - end - lead - trail;
            let size = clamp(if auto_size { available } else { preferred });
            let free = available - size;
            if auto_lead {
                lead += if !auto_trail {
                    free
                } else if inline && free < 0.0 {
                    0.0
                } else {
                    free / 2.0
                };
            }
            (start + lead, size)
        }
        (Some(start), None) => (start + lead, clamp(preferred)),
        (None, Some(end)) => {
            let size = clamp(preferred);
            (extent - end - trail - size, size)
        }
        (None, None) => (lead, clamp(preferred)),
    };
    (
        if reversed {
            extent - offset - size
        } else {
            offset
        },
        size,
    )
}

impl Default for Inset {
    /// The initial value: all edges `auto`.
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
        // when the width is automatic.
        let inset = Inset::new(
            Dimension::Px(10.0),
            Dimension::Px(30.0),
            Dimension::Px(10.0),
            Dimension::Px(10.0),
        );
        let rect = inset.resolve_absolute_box(
            CB,
            Dimension::Auto,
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
