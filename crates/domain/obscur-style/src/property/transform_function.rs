#![doc = include_str!("transform_function.doc/_module.md")]

use crate::property::{LengthContext, LengthExpr};

#[doc = include_str!("transform_function.doc/TransformFunction.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum TransformFunction {
    #[doc = include_str!("transform_function.doc/Matrix.md")]
    Matrix([f32; 6]),
    #[doc = include_str!("transform_function.doc/Translate.md")]
    Translate(LengthExpr, LengthExpr),
}

impl TransformFunction {
    #[doc = include_str!("transform_function.doc/uses_ch.md")]
    #[must_use]
    pub fn uses_ch(&self) -> bool {
        match self {
            Self::Matrix(_) => false,
            Self::Translate(x, y) => x.uses_ch() || y.uses_ch(),
        }
    }

    #[doc = include_str!("transform_function.doc/absolutize.md")]
    #[must_use]
    pub fn absolutize(&self, ctx: &LengthContext) -> Self {
        match self {
            Self::Matrix(coeffs) => Self::Matrix(*coeffs),
            Self::Translate(x, y) => Self::Translate(x.absolutize(ctx), y.absolutize(ctx)),
        }
    }

    #[doc = include_str!("transform_function.doc/resolve.md")]
    #[must_use]
    pub fn resolve(&self, ctx: &LengthContext) -> Option<[f32; 6]> {
        match self {
            Self::Matrix(coeffs) => Some(*coeffs),
            Self::Translate(x, y) => Some([1.0, 0.0, 0.0, 1.0, x.resolve(ctx)?, y.resolve(ctx)?]),
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
