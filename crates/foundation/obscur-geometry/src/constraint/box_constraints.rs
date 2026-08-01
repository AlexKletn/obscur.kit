#![doc = include_str!("box_constraints.doc/_module.md")]

use crate::primitive::Size;

#[doc = include_str!("box_constraints.doc/BoxConstraints.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxConstraints {
    #[doc = include_str!("box_constraints.doc/min.md")]
    pub min: Size,
    #[doc = include_str!("box_constraints.doc/max.md")]
    pub max: Size,
    #[doc = include_str!("box_constraints.doc/percent_basis.md")]
    pub percent_basis: Option<Size>,
}

impl BoxConstraints {
    #[doc = include_str!("box_constraints.doc/loose.md")]
    #[must_use]
    pub const fn loose(max: Size) -> Self {
        Self {
            min: Size::ZERO,
            max,
            percent_basis: None,
        }
    }

    #[doc = include_str!("box_constraints.doc/with_percent_basis.md")]
    #[must_use]
    pub const fn with_percent_basis(mut self, basis: Size) -> Self {
        self.percent_basis = Some(basis);
        self
    }

    #[doc = include_str!("box_constraints.doc/percent_width.md")]
    #[must_use]
    pub fn percent_width(self) -> f32 {
        self.percent_basis
            .map_or(self.max.width, |basis| basis.width)
    }

    #[doc = include_str!("box_constraints.doc/percent_height.md")]
    #[must_use]
    pub fn percent_height(self) -> f32 {
        self.percent_basis
            .map_or(self.max.height, |basis| basis.height)
    }

    #[doc = include_str!("box_constraints.doc/tight.md")]
    #[must_use]
    pub const fn tight(size: Size) -> Self {
        Self {
            min: size,
            max: size,
            percent_basis: None,
        }
    }

    #[doc = include_str!("box_constraints.doc/constrain.md")]
    #[must_use]
    pub fn constrain(self, desired: Size) -> Size {
        Size::new(
            desired.width.clamp(self.min.width, self.max.width),
            desired.height.clamp(self.min.height, self.max.height),
        )
    }

    #[doc = include_str!("box_constraints.doc/constrain_allowing_overflow.md")]
    #[must_use]
    pub fn constrain_allowing_overflow(self, desired: Size) -> Size {
        Size::new(
            resolve_axis(desired.width, self.min.width, self.max.width),
            resolve_axis(desired.height, self.min.height, self.max.height),
        )
    }
}

#[doc = include_str!("box_constraints.doc/resolve_axis.md")]
fn resolve_axis(desired: f32, min: f32, max: f32) -> f32 {
    if min >= max && max > 0.0 {
        max
    } else {
        desired.max(min)
    }
}

#[cfg(test)]
mod tests {
    use super::BoxConstraints;
    use crate::primitive::Size;

    #[test]
    fn a_tight_axis_dictates_and_ignores_what_the_box_wanted() {
        // How an absolutely-positioned box with both insets, and the root under
        // the viewport, are sized: the caller is assigning, not offering.
        let c = BoxConstraints::tight(Size::new(260.0, 300.0));
        assert_eq!(
            c.constrain_allowing_overflow(Size::new(999.0, 999.0)),
            Size::new(260.0, 300.0)
        );
        assert_eq!(
            c.constrain_allowing_overflow(Size::new(1.0, 1.0)),
            Size::new(260.0, 300.0)
        );
    }

    #[test]
    fn a_loose_axis_lets_content_overflow_the_space_offered() {
        // THE difference from `constrain`: 86.4 of unbreakable content offered
        // 40px reports 86.4, because that is what it needs.
        let c = BoxConstraints::loose(Size::new(40.0, 400.0));
        let got = c.constrain_allowing_overflow(Size::new(86.4, 10.0));
        assert_eq!(got.width, 86.4);
        assert_eq!(
            c.constrain(Size::new(86.4, 10.0)).width,
            40.0,
            "`constrain` reports the space offered — the answer that was never true"
        );
    }

    #[test]
    fn a_loose_axis_still_floors_at_the_minimum() {
        let c = BoxConstraints {
            min: Size::new(50.0, 5.0),
            max: Size::new(400.0, 400.0),
            percent_basis: None,
        };
        assert_eq!(
            c.constrain_allowing_overflow(Size::new(10.0, 1.0)),
            Size::new(50.0, 5.0)
        );
    }

    #[test]
    fn content_that_fits_is_unaffected() {
        // The overwhelmingly common case: the two agree exactly, so nothing that
        // fits its container changes behaviour.
        let c = BoxConstraints::loose(Size::new(400.0, 400.0));
        let desired = Size::new(120.0, 33.0);
        assert_eq!(c.constrain_allowing_overflow(desired), desired);
        assert_eq!(c.constrain_allowing_overflow(desired), c.constrain(desired));
    }

    #[test]
    fn a_zero_offer_lets_content_report_what_it_needs() {
        // The `min-content` probe. `loose(0)` and `tight(0)` are the same value,
        // so this axis is read as "no space available" rather than "be zero" —
        // otherwise every min-content answer would be zero. See the method's note.
        let c = BoxConstraints::loose(Size::new(0.0, 400.0));
        assert_eq!(
            c.constrain_allowing_overflow(Size::new(86.4, 48.0)),
            Size::new(86.4, 48.0),
            "a zero offer must not be mistaken for a zero assignment"
        );
    }

    #[test]
    fn an_unbounded_axis_returns_what_was_wanted() {
        // A `max-content` probe: infinite space offered, a finite want reported.
        let c = BoxConstraints::loose(Size::new(f32::INFINITY, f32::INFINITY));
        assert_eq!(
            c.constrain_allowing_overflow(Size::new(201.6, 16.0)),
            Size::new(201.6, 16.0)
        );
    }
}
