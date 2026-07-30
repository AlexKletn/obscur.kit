//! `ChildPercentBasis`: the containing block one child's percentages resolve
//! against, stated per axis by the algorithm that arranged it.

use obscur_geometry::Size;

/// What a **child's** percentage size resolves against, as the algorithm that
/// placed that child states it — one optional component per axis.
///
/// # Why the runtime cannot work this out for itself
///
/// The runtime's arrange walk hands every child of a node the same percentage
/// basis: the node's own content box. That is the containing block for a block
/// child (CSS 2.1 §10.1) and for a flex item (css-flexbox-1 §4), so for those
/// formatting contexts it is exactly right and this type is never stated.
///
/// It is **not** right for a grid item. `specs/w3c/css-grid-2.txt:712`, vendored
/// here: "A grid item's grid area forms the containing block into which it is
/// laid out", restated at `:1042` — "A grid item is sized within the containing
/// block defined by its grid area." The grid area is a per-item rectangle carved
/// out of the resolved tracks; only the grid algorithm knows it, and it differs
/// from the container's content box for every grid that has more than one track.
/// Handing the container's content box down instead is how a `width: 100%` grid
/// item came out at the track width (correct) while its children resolved their
/// percentages against the whole grid container's content width — the item's used
/// size and the size its children saw disagreed
/// (`docs/external-consumer-audit.md`, 2-4 (a)).
///
/// # Why per axis, and not a [`Size`]
///
/// [`BoxConstraints::percent_basis`] is a single [`Size`], so a caller either
/// states both axes or neither — which is why [`FlowLayout`] attaches its basis
/// only when its *block* axis is definite and lets a perfectly well-known inline
/// basis fall back with it. A grid is the mirror image: an item's grid area has a
/// resolved inline extent by the time any item is laid out, while its block
/// extent may be the item's own content (an `auto` row is sized *by* the items in
/// it), and a containing block derived from the box it contains is no containing
/// block at all. Stating one axis and leaving the other alone is the only honest
/// answer, and it is what keeps a fix in one axis from silently moving the other.
///
/// [`BoxConstraints::percent_basis`]: obscur_geometry::BoxConstraints::percent_basis
/// [`FlowLayout`]: super::FlowLayout
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ChildPercentBasis {
    /// The containing block's content **width**, when the algorithm knows it.
    pub width: Option<f32>,
    /// The containing block's content **height**, when the algorithm knows it.
    pub height: Option<f32>,
}

impl ChildPercentBasis {
    /// Nothing stated: both axes fall back to whatever the caller already had.
    /// The value every algorithm that is not grid produces, so their arranged
    /// children are byte-for-byte unchanged.
    pub const UNSTATED: Self = Self {
        width: None,
        height: None,
    };

    /// States the containing block's content width only, leaving the block axis
    /// to the caller's fallback.
    #[must_use]
    pub const fn with_width(width: f32) -> Self {
        Self {
            width: Some(width),
            height: None,
        }
    }

    /// Fills in the axes this basis leaves unstated from `fallback` — the basis
    /// the caller would have used on its own.
    #[must_use]
    pub fn resolve(self, fallback: Size) -> Size {
        Size::new(
            self.width.unwrap_or(fallback.width),
            self.height.unwrap_or(fallback.height),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ChildPercentBasis;
    use obscur_geometry::Size;

    #[test]
    fn unstated_defers_both_axes_to_the_fallback() {
        let fallback = Size::new(432.0, 192.0);
        assert_eq!(ChildPercentBasis::UNSTATED.resolve(fallback), fallback);
        assert_eq!(ChildPercentBasis::default(), ChildPercentBasis::UNSTATED);
    }

    #[test]
    fn a_stated_inline_axis_overrides_only_that_axis() {
        // The audit's `100%` row: the item's grid area is 211 wide, the grid
        // container's content box 432×192. The inline axis becomes the area; the
        // block axis is left exactly as the caller had it.
        let got = ChildPercentBasis::with_width(211.0).resolve(Size::new(432.0, 192.0));
        assert_eq!(got, Size::new(211.0, 192.0));
    }
}
