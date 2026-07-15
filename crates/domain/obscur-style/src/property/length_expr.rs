//! A symbolic length expression: [`LengthExpr`].
//!
//! `calc()`, `min()`, `max()` and `clamp()` combine several length terms with
//! arithmetic and folding rules (spec §4.13, "Supported"). Rather than evaluate
//! them at parse time — when the font size and viewport are still unknown — the
//! compiler lowers them into this symbolic tree, which is resolved to concrete
//! logical pixels at the layout/compute step against a
//! [`LengthContext`](crate::property::LengthContext) ("resolve at use time").
//!
//! The leaves are [`Length`] terms (`px` / `%` / `em` / `rem` / `vw` / `vh` /
//! …), so an expression can freely mix absolute, font-relative and
//! viewport-relative units. Multiplication and division follow the CSS rule that
//! one operand is a plain number: the coefficient is carried on the
//! [`LengthExpr::Product`] / [`LengthExpr::Quotient`] node.

use crate::property::{Length, LengthContext};

/// A `calc()` / `min()` / `max()` / `clamp()` expression over length terms.
///
/// Every node resolves to logical pixels via [`LengthExpr::resolve`]; a term
/// that cannot fold against the context (for example a `%` with no known basis,
/// or `auto`) makes the whole expression resolve to [`None`].
///
/// Because the leaves carry `f32` data, this derives [`PartialEq`] but not
/// [`Eq`]/[`Hash`].
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

    /// Returns `true` when any leaf anywhere in this expression is a `ch` term.
    ///
    /// Exhaustive over the tree, because `ch` is not required to be at the top:
    /// `clamp(10px, 20ch, 50%)` needs the `0` glyph measured just as much as a
    /// bare `20ch` does. See [`Length::is_ch`].
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

    /// Resolves this expression to concrete logical pixels against `ctx`.
    ///
    /// Returns [`None`] when any term cannot be folded (an `auto` term, a `%`
    /// with no known [`percent_basis`](crate::property::LengthContext), a
    /// division by zero, or an empty `min()` / `max()` list).
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

/// Resolves every argument and folds them with `combine`, returning [`None`] if
/// the list is empty or any argument fails to resolve.
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
