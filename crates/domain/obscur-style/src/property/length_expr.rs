#![doc = include_str!("../../docs/api/property/length_expr.doc/_module.md")]

use crate::property::{Length, LengthContext};

#[doc = include_str!("../../docs/api/property/length_expr.doc/LengthExpr.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum LengthExpr {
    /// A single length term (`10px`, `50%`, `2em`, `5vw`, …).
    Leaf(Length),
    /// The sum of two sub-expressions (`a + b`).
    Sum(Box<LengthExpr>, Box<LengthExpr>),
    /// The difference of two sub-expressions (`a - b`).
    Diff(Box<LengthExpr>, Box<LengthExpr>),
    /// A sub-expression scaled by a plain number (`a * n`).
    Product(Box<LengthExpr>, f32),
    /// A sub-expression divided by a plain number (`a / n`).
    Quotient(Box<LengthExpr>, f32),
    /// `min(a, b, …)`: the smallest of its arguments.
    Min(Vec<LengthExpr>),
    /// `max(a, b, …)`: the largest of its arguments.
    Max(Vec<LengthExpr>),
    /// `clamp(min, val, max)`: `val` bounded to the `[min, max]` range,
    /// evaluated as `max(min, min(val, max))`.
    Clamp(Box<LengthExpr>, Box<LengthExpr>, Box<LengthExpr>),
}

impl LengthExpr {
    /// Wraps a single length term as a leaf expression.
    #[must_use]
    pub const fn leaf(length: Length) -> Self {
        Self::Leaf(length)
    }

    #[doc = include_str!("../../docs/api/property/length_expr.doc/uses_ch.md")]
    #[must_use]
    pub fn uses_ch(&self) -> bool {
        match self {
            Self::Leaf(length) => length.is_ch(),
            Self::Product(inner, _) | Self::Quotient(inner, _) => inner.uses_ch(),
            Self::Sum(a, b) | Self::Diff(a, b) => a.uses_ch() || b.uses_ch(),
            Self::Min(terms) | Self::Max(terms) => terms.iter().any(Self::uses_ch),
            Self::Clamp(a, b, c) => a.uses_ch() || b.uses_ch() || c.uses_ch(),
        }
    }

    #[doc = include_str!("../../docs/api/property/length_expr.doc/uses_percent.md")]
    #[must_use]
    pub fn uses_percent(&self) -> bool {
        match self {
            Self::Leaf(length) => matches!(length, Length::Percent(_)),
            Self::Product(inner, _) | Self::Quotient(inner, _) => inner.uses_percent(),
            Self::Sum(a, b) | Self::Diff(a, b) => a.uses_percent() || b.uses_percent(),
            Self::Min(terms) | Self::Max(terms) => terms.iter().any(Self::uses_percent),
            Self::Clamp(a, b, c) => a.uses_percent() || b.uses_percent() || c.uses_percent(),
        }
    }

    pub(crate) fn only_percentages(&self) -> bool {
        match self {
            Self::Leaf(length) => matches!(length, Length::Percent(_)),
            Self::Product(inner, _) | Self::Quotient(inner, _) => inner.only_percentages(),
            Self::Sum(a, b) | Self::Diff(a, b) => a.only_percentages() && b.only_percentages(),
            Self::Min(terms) | Self::Max(terms) => terms.iter().all(Self::only_percentages),
            Self::Clamp(a, b, c) => {
                a.only_percentages() && b.only_percentages() && c.only_percentages()
            }
        }
    }

    #[doc = include_str!("../../docs/api/property/length_expr.doc/absolutize.md")]
    #[must_use]
    pub fn absolutize(&self, ctx: &LengthContext) -> Self {
        match self {
            Self::Leaf(length) => Self::Leaf(length.absolutize(ctx)),
            Self::Sum(a, b) => Self::Sum(Box::new(a.absolutize(ctx)), Box::new(b.absolutize(ctx))),
            Self::Diff(a, b) => {
                Self::Diff(Box::new(a.absolutize(ctx)), Box::new(b.absolutize(ctx)))
            }
            Self::Product(a, n) => Self::Product(Box::new(a.absolutize(ctx)), *n),
            Self::Quotient(a, n) => Self::Quotient(Box::new(a.absolutize(ctx)), *n),
            Self::Min(args) => Self::Min(args.iter().map(|a| a.absolutize(ctx)).collect()),
            Self::Max(args) => Self::Max(args.iter().map(|a| a.absolutize(ctx)).collect()),
            Self::Clamp(min, val, max) => Self::Clamp(
                Box::new(min.absolutize(ctx)),
                Box::new(val.absolutize(ctx)),
                Box::new(max.absolutize(ctx)),
            ),
        }
    }

    #[doc = include_str!("../../docs/api/property/length_expr.doc/resolve.md")]
    #[must_use]
    pub fn resolve(&self, ctx: &LengthContext) -> Option<f32> {
        self.resolve_with(&|length| length.resolve_px(ctx))
    }

    /// Returns the pixel offset and fractional percentage coefficient of a linear length expression.
    #[must_use]
    pub fn affine_components(&self, ctx: &LengthContext) -> Option<(f32, f32)> {
        let lower = |term: &Self| term.affine_components(ctx);
        let (pixels, fraction) = match self {
            Self::Leaf(Length::Percent(value)) => (0.0, value / 100.0),
            Self::Leaf(length) => (length.resolve_px(ctx)?, 0.0),
            Self::Sum(a, b) | Self::Diff(a, b) => {
                let (a_px, a_fraction) = lower(a)?;
                let (b_px, b_fraction) = lower(b)?;
                let sign = if matches!(self, Self::Sum(..)) {
                    1.0
                } else {
                    -1.0
                };
                (a_px + sign * b_px, a_fraction + sign * b_fraction)
            }
            Self::Product(term, scalar) | Self::Quotient(term, scalar) => {
                let (px, fraction) = lower(term)?;
                let factor = if matches!(self, Self::Product(..)) {
                    *scalar
                } else {
                    scalar.recip()
                };
                (px * factor, fraction * factor)
            }
            expression if !expression.uses_percent() => (expression.resolve(ctx)?, 0.0),
            _ => return None,
        };
        (pixels.is_finite() && fraction.is_finite()).then_some((pixels, fraction))
    }

    /// Resolves a computed expression with pixel and percentage leaves against an explicit basis.
    #[must_use]
    pub fn resolve_with_percentage_basis(&self, basis: f32) -> Option<f32> {
        let value = self.resolve_with(&|length| match length {
            Length::Px(px) => Some(px.0),
            Length::Percent(percent) if basis.is_finite() => Some(obscur_geometry::saturating_f32(
                f64::from(percent) * f64::from(basis) / 100.0,
            )),
            _ => None,
        })?;
        (!value.is_nan()).then(|| obscur_geometry::saturating_f32(f64::from(value)))
    }

    pub(crate) fn resolve_with(
        &self,
        resolve_leaf: &impl Fn(Length) -> Option<f32>,
    ) -> Option<f32> {
        match self {
            Self::Leaf(length) => resolve_leaf(*length),
            Self::Sum(a, b) => Some(a.resolve_with(resolve_leaf)? + b.resolve_with(resolve_leaf)?),
            Self::Diff(a, b) => Some(a.resolve_with(resolve_leaf)? - b.resolve_with(resolve_leaf)?),
            Self::Product(a, n) => Some(a.resolve_with(resolve_leaf)? * n),
            Self::Quotient(a, n) => {
                if *n == 0.0 {
                    None
                } else {
                    Some(a.resolve_with(resolve_leaf)? / n)
                }
            }
            Self::Min(args) => fold(args, resolve_leaf, f32::min),
            Self::Max(args) => fold(args, resolve_leaf, f32::max),
            Self::Clamp(min, val, max) => {
                let min = min.resolve_with(resolve_leaf)?;
                let val = val.resolve_with(resolve_leaf)?;
                let max = max.resolve_with(resolve_leaf)?;
                Some(min.max(val.min(max)))
            }
        }
    }
}

/// Resolves every argument and folds them with `combine`, returning [`None`] if
/// the list is empty or any argument fails to resolve.
fn fold(
    args: &[LengthExpr],
    resolve_leaf: &impl Fn(Length) -> Option<f32>,
    combine: fn(f32, f32) -> f32,
) -> Option<f32> {
    let mut iter = args.iter();
    let mut acc = iter.next()?.resolve_with(resolve_leaf)?;
    for arg in iter {
        acc = combine(acc, arg.resolve_with(resolve_leaf)?);
    }
    Some(acc)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> LengthContext {
        LengthContext::new(16.0, 20.0, 800.0, 600.0)
    }

    #[test]
    fn uses_ch_finds_a_ch_leaf_at_any_depth() {
        // The point of recursing: `ch` is rarely the top node of the tree.
        let deep = LengthExpr::Clamp(
            Box::new(LengthExpr::leaf(Length::px(10.0))),
            Box::new(LengthExpr::Sum(
                Box::new(LengthExpr::leaf(Length::px(2.0))),
                Box::new(LengthExpr::Product(
                    Box::new(LengthExpr::leaf(Length::Ch(3.0))),
                    2.0,
                )),
            )),
            Box::new(LengthExpr::leaf(Length::Percent(50.0))),
        );
        assert!(deep.uses_ch());
        assert!(
            LengthExpr::Min(vec![
                LengthExpr::leaf(Length::px(1.0)),
                LengthExpr::leaf(Length::Ch(1.0)),
            ])
            .uses_ch()
        );
    }

    #[test]
    fn uses_ch_is_false_for_an_expression_of_other_units() {
        // Must not fire on `em`: the two are different bases, and a false
        // positive would cost a shaping call on every ordinary stylesheet.
        let expr = LengthExpr::Sum(
            Box::new(LengthExpr::leaf(Length::Em(2.0))),
            Box::new(LengthExpr::Max(vec![
                LengthExpr::leaf(Length::Rem(1.0)),
                LengthExpr::leaf(Length::Vw(50.0)),
            ])),
        );
        assert!(!expr.uses_ch());
    }

    #[test]
    fn uses_percent_finds_a_percent_leaf_at_any_depth() {
        // `transform: translateX(calc(100% - 10px))` must be refused, not
        // resolved with the `%` quietly taken as `0`.
        let deep = LengthExpr::Diff(
            Box::new(LengthExpr::leaf(Length::Percent(100.0))),
            Box::new(LengthExpr::leaf(Length::px(10.0))),
        );
        assert!(deep.uses_percent());
        assert!(
            LengthExpr::Clamp(
                Box::new(LengthExpr::leaf(Length::px(1.0))),
                Box::new(LengthExpr::Product(
                    Box::new(LengthExpr::leaf(Length::Percent(50.0))),
                    2.0,
                )),
                Box::new(LengthExpr::leaf(Length::px(9.0))),
            )
            .uses_percent()
        );
        // Must not fire on the units a length context CAN resolve.
        assert!(
            !LengthExpr::Min(vec![
                LengthExpr::leaf(Length::Em(1.0)),
                LengthExpr::leaf(Length::Vw(2.0)),
            ])
            .uses_percent()
        );
    }

    #[test]
    fn absolutize_folds_relative_leaves_and_keeps_the_shape() {
        // calc(50% - 2em) computes to calc(50% - 32px): the `em` is gone, the
        // `%` survives for layout to resolve against the containing block.
        let expr = LengthExpr::Diff(
            Box::new(LengthExpr::leaf(Length::Percent(50.0))),
            Box::new(LengthExpr::leaf(Length::Em(2.0))),
        );
        assert_eq!(
            expr.absolutize(&ctx()),
            LengthExpr::Diff(
                Box::new(LengthExpr::leaf(Length::Percent(50.0))),
                Box::new(LengthExpr::leaf(Length::px(32.0))),
            )
        );
    }

    #[test]
    fn absolutize_reaches_every_leaf_of_the_tree() {
        let deep = LengthExpr::Clamp(
            Box::new(LengthExpr::leaf(Length::Rem(1.0))),
            Box::new(LengthExpr::Min(vec![
                LengthExpr::Product(Box::new(LengthExpr::leaf(Length::Em(1.0))), 2.0),
                LengthExpr::Quotient(Box::new(LengthExpr::leaf(Length::Vw(50.0))), 2.0),
            ])),
            Box::new(LengthExpr::Sum(
                Box::new(LengthExpr::leaf(Length::Vh(10.0))),
                Box::new(LengthExpr::Max(vec![LengthExpr::leaf(Length::Vmin(10.0))])),
            )),
        );
        // Root 20, font 16, viewport 800x600 -> clamp(20px, min(32px, 400px/2),
        // 60px + max(60px)); every leaf is now `px`, so resolving needs no bases.
        let absolute = deep.absolutize(&ctx());
        assert_eq!(
            absolute.resolve(&LengthContext::new(0.0, 0.0, 0.0, 0.0)),
            deep.resolve(&ctx())
        );
    }

    #[test]
    fn a_ch_term_folds_inside_an_expression() {
        // 3ch against a measured 10px advance, plus 12px.
        let expr = LengthExpr::Sum(
            Box::new(LengthExpr::leaf(Length::Ch(3.0))),
            Box::new(LengthExpr::leaf(Length::px(12.0))),
        );
        assert_eq!(expr.resolve(&ctx().with_ch_width(10.0)), Some(42.0));
    }

    #[allow(clippy::unnecessary_box_returns)] // test convenience: builds boxed operands directly
    fn px(v: f32) -> Box<LengthExpr> {
        Box::new(LengthExpr::Leaf(Length::px(v)))
    }

    #[test]
    fn leaf_resolves_like_its_length() {
        assert_eq!(
            LengthExpr::Leaf(Length::px(10.0)).resolve(&ctx()),
            Some(10.0)
        );
        assert_eq!(
            LengthExpr::Leaf(Length::Em(2.0)).resolve(&ctx()),
            Some(32.0)
        );
    }

    #[test]
    fn calc_sum_mixes_units() {
        // calc(10px + 2em) with font-size 16 -> 10 + 32 = 42.
        let expr = LengthExpr::Sum(px(10.0), Box::new(LengthExpr::Leaf(Length::Em(2.0))));
        assert_eq!(expr.resolve(&ctx()), Some(42.0));
    }

    #[test]
    fn calc_diff_and_product_and_quotient() {
        assert_eq!(
            LengthExpr::Diff(px(100.0), px(20.0)).resolve(&ctx()),
            Some(80.0)
        );
        assert_eq!(
            LengthExpr::Product(px(10.0), 3.0).resolve(&ctx()),
            Some(30.0)
        );
        assert_eq!(
            LengthExpr::Quotient(px(30.0), 3.0).resolve(&ctx()),
            Some(10.0)
        );
        // Division by zero is undefined and does not resolve.
        assert_eq!(LengthExpr::Quotient(px(30.0), 0.0).resolve(&ctx()), None);
    }

    #[test]
    fn min_and_max_fold_arguments() {
        let args = vec![
            LengthExpr::Leaf(Length::px(10.0)),
            LengthExpr::Leaf(Length::px(30.0)),
            LengthExpr::Leaf(Length::px(20.0)),
        ];
        assert_eq!(LengthExpr::Min(args.clone()).resolve(&ctx()), Some(10.0));
        assert_eq!(LengthExpr::Max(args).resolve(&ctx()), Some(30.0));
        assert_eq!(LengthExpr::Min(vec![]).resolve(&ctx()), None);
    }

    #[test]
    fn clamp_bounds_the_value() {
        // clamp(10px, 5vw, 100px): 5vw = 40 -> within [10,100] -> 40.
        let expr = LengthExpr::Clamp(
            px(10.0),
            Box::new(LengthExpr::Leaf(Length::Vw(5.0))),
            px(100.0),
        );
        assert_eq!(expr.resolve(&ctx()), Some(40.0));
        // clamp(10px, 5px, 100px): below min -> 10.
        let low = LengthExpr::Clamp(px(10.0), px(5.0), px(100.0));
        assert_eq!(low.resolve(&ctx()), Some(10.0));
        // clamp(10px, 500px, 100px): above max -> 100.
        let high = LengthExpr::Clamp(px(10.0), px(500.0), px(100.0));
        assert_eq!(high.resolve(&ctx()), Some(100.0));
    }

    #[test]
    fn unresolvable_term_poisons_the_expression() {
        // A bare percentage with no basis cannot fold, so calc does not resolve.
        let expr = LengthExpr::Sum(px(10.0), Box::new(LengthExpr::Leaf(Length::Percent(50.0))));
        assert_eq!(expr.resolve(&ctx()), None);
    }
}
