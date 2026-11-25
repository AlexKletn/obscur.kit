//! `ChildPercentBasis`: the containing block one child's percentages resolve
//! against, stated per axis by the algorithm that arranged it.

use obscur_geometry::Size;

#[doc = include_str!("../../docs/api/algorithm/child_percent_basis.doc/ChildPercentBasis.md")]
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

    #[doc = include_str!("../../docs/api/algorithm/child_percent_basis.doc/with_size.md")]
    #[must_use]
    pub const fn with_size(width: f32, height: f32) -> Self {
        Self {
            width: Some(width),
            height: Some(height),
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
