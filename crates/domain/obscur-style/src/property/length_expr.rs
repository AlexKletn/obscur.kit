#![doc = include_str!("length_expr.doc/_module.md")]

use crate::property::{Length, LengthContext};

#[doc = include_str!("length_expr.doc/LengthExpr.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum LengthExpr {
    #[doc = include_str!("length_expr.doc/Leaf.md")]
    Leaf(Length),
    #[doc = include_str!("length_expr.doc/Sum.md")]
    Sum(Box<LengthExpr>, Box<LengthExpr>),
    #[doc = include_str!("length_expr.doc/Diff.md")]
    Diff(Box<LengthExpr>, Box<LengthExpr>),
    #[doc = include_str!("length_expr.doc/Product.md")]
    Product(Box<LengthExpr>, f32),
    #[doc = include_str!("length_expr.doc/Quotient.md")]
    Quotient(Box<LengthExpr>, f32),
    #[doc = include_str!("length_expr.doc/Min.md")]
    Min(Vec<LengthExpr>),
    #[doc = include_str!("length_expr.doc/Max.md")]
    Max(Vec<LengthExpr>),
    #[doc = include_str!("length_expr.doc/Clamp.md")]
    Clamp(Box<LengthExpr>, Box<LengthExpr>, Box<LengthExpr>),
}

impl LengthExpr {
    #[doc = include_str!("length_expr.doc/leaf.md")]
    #[must_use]
    pub const fn leaf(length: Length) -> Self {
        Self::Leaf(length)
    }

    #[doc = include_str!("length_expr.doc/uses_ch.md")]
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

    #[doc = include_str!("length_expr.doc/absolutize.md")]
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

    #[doc = include_str!("length_expr.doc/resolve.md")]
    #[must_use]
    pub fn resolve(&self, ctx: &LengthContext) -> Option<f32> {
        match self {
            Self::Leaf(length) => length.resolve_px(ctx),
            Self::Sum(a, b) => Some(a.resolve(ctx)? + b.resolve(ctx)?),
            Self::Diff(a, b) => Some(a.resolve(ctx)? - b.resolve(ctx)?),
            Self::Product(a, n) => Some(a.resolve(ctx)? * n),
            Self::Quotient(a, n) => {
                if *n == 0.0 {
                    None
                } else {
                    Some(a.resolve(ctx)? / n)
                }
            }
            Self::Min(args) => fold(args, ctx, f32::min),
            Self::Max(args) => fold(args, ctx, f32::max),
            Self::Clamp(min, val, max) => {
                let min = min.resolve(ctx)?;
                let val = val.resolve(ctx)?;
                let max = max.resolve(ctx)?;
                Some(min.max(val.min(max)))
            }
        }
    }
}

#[doc = include_str!("length_expr.doc/fold.md")]
fn fold(args: &[LengthExpr], ctx: &LengthContext, combine: fn(f32, f32) -> f32) -> Option<f32> {
    let mut iter = args.iter();
    let mut acc = iter.next()?.resolve(ctx)?;
    for arg in iter {
        acc = combine(acc, arg.resolve(ctx)?);
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
