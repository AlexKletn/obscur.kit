//! One entry of a specified `filter` / `backdrop-filter` list, before its lengths
//! are known (spec §4.13, §5.2).

use crate::property::{FilterOp, LengthContext, LengthExpr};

#[doc = include_str!("../../docs/api/property/filter_function.doc/FilterFunction.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum FilterFunction {
    /// A function already reduced to a [`FilterOp`] — every filter function except a
    /// `blur()` written in a relative unit.
    Op(FilterOp),
    /// A `blur(<length>)` whose radius still depends on the font or the viewport.
    Blur(LengthExpr),
}

impl FilterFunction {
    /// Whether this function is written in `ch`, which needs a measured `0` advance and
    /// so defers the whole computed value until the font is measured.
    #[must_use]
    pub fn uses_ch(&self) -> bool {
        match self {
            Self::Op(op) => op.uses_ch(),
            Self::Blur(radius) => radius.uses_ch(),
        }
    }

    /// Folds the `blur()` radius as far as this length context reaches, keeping the
    /// rest symbolic; an already-resolved op is returned unchanged (its own lengths —
    /// `drop-shadow()`'s offsets — are folded by the computed-value pass that owns
    /// [`FilterOp`]).
    #[must_use]
    pub fn absolutize_blur(&self, ctx: &LengthContext) -> Self {
        match self {
            Self::Op(op) => Self::Op(op.clone()),
            Self::Blur(radius) => Self::Blur(radius.absolutize(ctx)),
        }
    }

    #[doc = include_str!("../../docs/api/property/filter_function.doc/resolve_blur_radius.md")]
    #[must_use]
    pub fn resolve_blur_radius(&self, ctx: &LengthContext) -> Option<f32> {
        match self {
            Self::Op(_) => None,
            Self::Blur(radius) => radius.resolve(ctx),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::Length;

    fn ctx() -> LengthContext {
        LengthContext::new(16.0, 20.0, 800.0, 600.0)
    }

    #[test]
    fn a_relative_blur_radius_resolves_against_the_context() {
        let blur = FilterFunction::Blur(LengthExpr::leaf(Length::Rem(1.0)));
        assert_eq!(blur.resolve_blur_radius(&ctx()), Some(20.0));
    }

    #[test]
    fn a_percentage_blur_radius_stays_unresolved() {
        // `blur()` takes a `<length>`; a `%` has no basis at all here, and the
        // answer is "not yet", never `0`.
        let blur = FilterFunction::Blur(LengthExpr::leaf(Length::Percent(50.0)));
        assert_eq!(blur.resolve_blur_radius(&ctx()), None);
    }

    #[test]
    fn a_resolved_op_rides_through_untouched() {
        let op = FilterFunction::Op(FilterOp::Grayscale(1.0));
        assert_eq!(op.absolutize_blur(&ctx()), op);
        assert_eq!(op.resolve_blur_radius(&ctx()), None);
        assert!(!op.uses_ch());
    }

    #[test]
    fn a_ch_radius_reports_itself() {
        assert!(FilterFunction::Blur(LengthExpr::leaf(Length::Ch(1.0))).uses_ch());
    }
}
