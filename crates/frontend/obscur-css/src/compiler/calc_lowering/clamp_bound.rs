use crate::parser::ParsedValue;

use super::{length_operand::LengthOperand, parse_full};

/// One of `clamp()`'s two **bound** arguments, once parsed.
pub(super) enum ClampBound {
    /// A specified clamp limit, represented as a number or length expression.
    Bounded(LengthOperand),
    /// The keyword `none`: the side is not clamped.
    None,
}

#[doc = include_str!("../../../docs/api/compiler/calc_lowering.doc/parse_clamp_bound.md")]
pub(super) fn parse_clamp_bound(tokens: &[ParsedValue]) -> Option<ClampBound> {
    if let [ParsedValue::Keyword(keyword)] = tokens
        && keyword.eq_ignore_ascii_case("none")
    {
        return Some(ClampBound::None);
    }
    parse_full(tokens).map(ClampBound::Bounded)
}
