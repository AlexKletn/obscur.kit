//! One entry of a specified `transform` function list, before its lengths are known
//! (spec §4.13).

use crate::property::{Length, LengthContext, LengthExpr};

#[doc = include_str!("../../docs/api/property/transform_function.doc/TransformFunction.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum TransformFunction {
    /// A function already reduced to its 2D affine coefficients `[a, b, c, d, e, f]`.
    Matrix([f32; 6]),
    /// A `translate(x, y)` whose distances are still expressions: the x offset, then
    /// the y offset (`translateX`/`translateY` fill the unused axis with `0`).
    Translate(LengthExpr, LengthExpr),
}

impl TransformFunction {
    /// Whether either distance is written in `ch`, which needs a measured `0` advance
    /// and so defers the whole computed value until the font is measured.
    #[must_use]
    pub fn uses_ch(&self) -> bool {
        match self {
            Self::Matrix(_) => false,
            Self::Translate(x, y) => x.uses_ch() || y.uses_ch(),
        }
    }

    /// Folds every distance that this length context can resolve, and keeps the rest
    /// symbolic — the same contract as [`Length::absolutize`](crate::property::Length::absolutize).
    #[must_use]
    pub fn absolutize(&self, ctx: &LengthContext) -> Self {
        match self {
            Self::Matrix(coeffs) => Self::Matrix(*coeffs),
            Self::Translate(x, y) => Self::Translate(x.absolutize(ctx), y.absolutize(ctx)),
        }
    }

    #[doc = include_str!("../../docs/api/property/transform_function.doc/resolve.md")]
    #[must_use]
    pub fn resolve(&self, ctx: &LengthContext) -> Option<[f32; 6]> {
        match self {
            Self::Matrix(coeffs) => Some(*coeffs),
            Self::Translate(x, y) => {
                let resolve = |expression: &LengthExpr| {
                    let value = expression.resolve(ctx)?;
                    (!value.is_nan()).then(|| obscur_geometry::saturating_f32(f64::from(value)))
                };
                Some([1.0, 0.0, 0.0, 1.0, resolve(x)?, resolve(y)?])
            }
        }
    }

    /// Resolves already-computed translation terms using the border box width and height.
    #[must_use]
    pub fn resolve_in_box(&self, width: f32, height: f32) -> Option<[f32; 6]> {
        let resolve = |expression: &LengthExpr, basis: f32| {
            let value = expression.resolve_with(&|length| match length {
                Length::Px(px) => Some(px.0),
                Length::Percent(percent) => Some(obscur_geometry::saturating_f32(
                    f64::from(percent) * f64::from(basis) / 100.0,
                )),
                _ => None,
            })?;
            (!value.is_nan()).then(|| obscur_geometry::saturating_f32(f64::from(value)))
        };
        match self {
            Self::Matrix(coeffs) => Some(*coeffs),
            Self::Translate(x, y) => {
                Some([1.0, 0.0, 0.0, 1.0, resolve(x, width)?, resolve(y, height)?])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::Length;

    fn ctx() -> LengthContext {
        // font 16, root 20 (distinct, so `em` and `rem` cannot be confused),
        // viewport 800x600.
        LengthContext::new(16.0, 20.0, 800.0, 600.0)
    }

    fn translate(x: Length, y: Length) -> TransformFunction {
        TransformFunction::Translate(LengthExpr::leaf(x), LengthExpr::leaf(y))
    }

    #[test]
    fn a_matrix_function_resolves_to_itself() {
        let m = TransformFunction::Matrix([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]);
        assert_eq!(m.resolve(&ctx()), Some([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]));
        assert_eq!(m.absolutize(&ctx()), m);
        assert!(!m.uses_ch());
    }

    #[test]
    fn a_relative_translate_resolves_against_the_context() {
        // 2em = 32 against a 16px font; 10vw = 80 against an 800px viewport.
        assert_eq!(
            translate(Length::Em(2.0), Length::Vw(10.0)).resolve(&ctx()),
            Some([1.0, 0.0, 0.0, 1.0, 32.0, 80.0])
        );
    }

    #[test]
    fn a_percentage_translate_stays_unresolved() {
        // The basis is the element's own border box, which no length context
        // carries: the answer is "not yet", never `0`.
        assert_eq!(
            translate(Length::Percent(50.0), Length::px(0.0)).resolve(&ctx()),
            None
        );
    }

    #[test]
    fn absolutizing_folds_what_it_can_and_keeps_what_it_cannot() {
        let folded = translate(Length::Rem(1.0), Length::Percent(50.0)).absolutize(&ctx());
        assert_eq!(folded, translate(Length::px(20.0), Length::Percent(50.0)));
    }

    #[test]
    fn a_ch_translate_reports_itself() {
        // `ch` needs a measured `0` advance, so the resolver defers the whole
        // value until the glyph is measured.
        assert!(translate(Length::Ch(2.0), Length::px(0.0)).uses_ch());
        assert!(!translate(Length::Em(2.0), Length::px(0.0)).uses_ch());
    }
}
