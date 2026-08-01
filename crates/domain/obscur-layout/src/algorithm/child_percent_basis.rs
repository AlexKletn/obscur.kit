#![doc = include_str!("child_percent_basis.doc/_module.md")]

use obscur_geometry::Size;

#[doc = include_str!("child_percent_basis.doc/ChildPercentBasis.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ChildPercentBasis {
    #[doc = include_str!("child_percent_basis.doc/width.md")]
    pub width: Option<f32>,
    #[doc = include_str!("child_percent_basis.doc/height.md")]
    pub height: Option<f32>,
}

impl ChildPercentBasis {
    #[doc = include_str!("child_percent_basis.doc/UNSTATED.md")]
    pub const UNSTATED: Self = Self {
        width: None,
        height: None,
    };

    #[doc = include_str!("child_percent_basis.doc/with_width.md")]
    #[must_use]
    pub const fn with_width(width: f32) -> Self {
        Self {
            width: Some(width),
            height: None,
        }
    }

    #[doc = include_str!("child_percent_basis.doc/with_size.md")]
    #[must_use]
    pub const fn with_size(width: f32, height: f32) -> Self {
        Self {
            width: Some(width),
            height: Some(height),
        }
    }

    #[doc = include_str!("child_percent_basis.doc/resolve.md")]
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
