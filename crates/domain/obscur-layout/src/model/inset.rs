//! The four positioning insets of a node (`top` / `right` / `bottom` / `left`).

use obscur_geometry::{Point, Rect, Size};

use crate::model::Dimension;

/// A node's positioning insets — the `top` / `right` / `bottom` / `left`
/// offsets used by a positioned node (spec §4.1; CSS `inset`).
///
/// Each edge is a [`Dimension`], so it distinguishes a definite offset
/// ([`Dimension::Px`] / [`Dimension::Percent`]) from `auto`
/// ([`Dimension::Auto`], the initial value). This is separate from the pixel
/// [`Insets`](obscur_geometry::Insets) used for margin/padding/border, which
/// cannot carry `auto` or percentages.
///
/// The insets only take effect for a [`Position::Relative`](crate::model::Position::Relative)
/// (or, later, out-of-flow) node; a [`Position::Static`](crate::model::Position::Static)
/// node ignores them entirely.
#[derive(Debug, Clone, Copy, PartialEq)]
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

    /// Resolves these insets into the absolute border-box [`Rect`] of an
    /// out-of-flow ([`Position::Absolute`](crate::model::Position::Absolute) /
    /// [`Position::Fixed`](crate::model::Position::Fixed)) node within its
    /// containing block `cb` (spec §4.1).
    ///
    /// `width` / `height` are the node's explicit size preferences and `natural`
    /// its measured natural (content) size; each is used when the corresponding
    /// axis is not fully pinned by both insets. Insets and explicit sizes resolve
    /// percentages against `cb`. Per axis (horizontal shown; vertical analogous
    /// with `top` / `bottom` / `height`):
    ///
    /// * **both `left` and `right` definite** → the box is pinned to both edges:
    ///   `x = cb.x + left`, `width = cb.width − left − right` (clamped ≥ 0). The
    ///   insets win; the explicit/natural width is ignored.
    /// * **only `left` definite** → `x = cb.x + left`, `width` = the explicit
    ///   width if set, else `natural.width`.
    /// * **only `right` definite** → `width` = explicit-or-natural, then
    ///   `x = cb.right − right − width`.
    /// * **neither definite** → **static-position fallback**: `x = cb.x` (a
    ///   documented simplification — the true static position would be the node's
    ///   in-flow position), `width` = explicit-or-natural.
    #[must_use]
    pub fn resolve_absolute_box(
        self,
        cb: Rect,
        width: Dimension,
        height: Dimension,
        natural: Size,
    ) -> Rect {
        let (x, w) = resolve_axis(
            self.left.resolve(cb.size.width),
            self.right.resolve(cb.size.width),
            width.resolve(cb.size.width),
            cb.origin.x,
            cb.size.width,
            natural.width,
        );
        let (y, h) = resolve_axis(
            self.top.resolve(cb.size.height),
            self.bottom.resolve(cb.size.height),
            height.resolve(cb.size.height),
            cb.origin.y,
            cb.size.height,
            natural.height,
        );
        Rect::new(Point::new(x, y), Size::new(w, h))
    }
}

/// Resolves one axis of an out-of-flow box from its start/end insets, explicit
/// size, containing-block origin/extent, and natural size, returning
/// `(position, size)` along that axis. See
/// [`Inset::resolve_absolute_box`] for the branch semantics.
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
    /// The initial value: all edges `auto`.
    fn default() -> Self {
        Self::AUTO
    }
}

#[cfg(test)]
mod tests {
    use super::Inset;
    use crate::model::Dimension;
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
        );
        assert_eq!(rect, Rect::new(Point::new(120.0, 110.0), Size::new(40.0, 30.0)));
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
        );
        // 100 + 200 - 0 - 20 = 280 on each axis.
        assert_eq!(rect, Rect::new(Point::new(280.0, 280.0), Size::new(20.0, 20.0)));
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
        );
        assert_eq!(rect, Rect::new(Point::new(100.0, 100.0), Size::new(40.0, 30.0)));
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
        );
        assert_eq!(rect.origin, Point::new(150.0, 120.0));
    }
}
