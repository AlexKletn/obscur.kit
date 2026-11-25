use obscur_style::property::LengthExpr;

/// One operand while folding a product/sum: either a resolved length
/// sub-expression or a plain (unitless) number.
pub(super) enum LengthOperand {
    /// A length sub-expression.
    Len(LengthExpr),
    /// A bare number, valid only as a `*` / `/` coefficient.
    Num(f32),
}

impl LengthOperand {
    /// Converts this operand into a length expression, failing for a bare number
    /// (a math function used as a length must yield a length, not a scalar).
    pub(super) fn into_length(self) -> Option<LengthExpr> {
        match self {
            Self::Len(expr) => Some(expr),
            Self::Num(_) => None,
        }
    }
}
