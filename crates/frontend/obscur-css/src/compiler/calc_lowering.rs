#![doc = include_str!("calc_lowering.doc/_module.md")]

use super::value_lowering::dimension_length;
use crate::parser::ParsedValue;

use obscur_style::property::{Length, LengthExpr};

#[doc = include_str!("calc_lowering.doc/Operand.md")]
enum Operand {
    #[doc = include_str!("calc_lowering.doc/Len.md")]
    Len(LengthExpr),
    #[doc = include_str!("calc_lowering.doc/Num.md")]
    Num(f32),
}

impl Operand {
    #[doc = include_str!("calc_lowering.doc/into_length.md")]
    fn into_length(self) -> Option<LengthExpr> {
        match self {
            Self::Len(expr) => Some(expr),
            Self::Num(_) => None,
        }
    }
}

#[doc = include_str!("calc_lowering.doc/lower_math_function.md")]
pub(crate) fn lower_math_function(name: &str, args: &[ParsedValue]) -> Option<LengthExpr> {
    if name.eq_ignore_ascii_case("calc") || name.is_empty() {
        parse_full(args)
    } else if name.eq_ignore_ascii_case("min") {
        Some(LengthExpr::Min(parse_arg_list(args)?))
    } else if name.eq_ignore_ascii_case("max") {
        Some(LengthExpr::Max(parse_arg_list(args)?))
    } else if name.eq_ignore_ascii_case("clamp") {
        let parts = split_top_level_commas(args);
        if parts.len() != 3 {
            return None;
        }
        let min = parse_clamp_bound(parts[0])?;
        let val = parse_full(parts[1])?;
        let max = parse_clamp_bound(parts[2])?;
        Some(match (min, max) {
            (ClampBound::Bounded(min), ClampBound::Bounded(max)) => {
                LengthExpr::Clamp(Box::new(min), Box::new(val), Box::new(max))
            }
            (ClampBound::Bounded(min), ClampBound::None) => LengthExpr::Max(vec![min, val]),
            (ClampBound::None, ClampBound::Bounded(max)) => LengthExpr::Min(vec![val, max]),
            (ClampBound::None, ClampBound::None) => val,
        })
    } else {
        None
    }
}

#[doc = include_str!("calc_lowering.doc/ClampBound.md")]
enum ClampBound {
    #[doc = include_str!("calc_lowering.doc/Bounded.md")]
    Bounded(LengthExpr),
    #[doc = include_str!("calc_lowering.doc/None.md")]
    None,
}

#[doc = include_str!("calc_lowering.doc/parse_clamp_bound.md")]
fn parse_clamp_bound(tokens: &[ParsedValue]) -> Option<ClampBound> {
    if let [ParsedValue::Keyword(keyword)] = tokens
        && keyword.eq_ignore_ascii_case("none")
    {
        return Some(ClampBound::None);
    }
    parse_full(tokens).map(ClampBound::Bounded)
}

#[doc = include_str!("calc_lowering.doc/parse_arg_list.md")]
fn parse_arg_list(args: &[ParsedValue]) -> Option<Vec<LengthExpr>> {
    let parts = split_top_level_commas(args);
    if parts.len() == 1 && parts[0].is_empty() {
        return None;
    }
    parts.into_iter().map(parse_full).collect()
}

#[doc = include_str!("calc_lowering.doc/split_top_level_commas.md")]
fn split_top_level_commas(args: &[ParsedValue]) -> Vec<&[ParsedValue]> {
    let mut parts = Vec::new();
    let mut start = 0;
    for (i, value) in args.iter().enumerate() {
        if matches!(value, ParsedValue::Comma) {
            parts.push(&args[start..i]);
            start = i + 1;
        }
    }
    parts.push(&args[start..]);
    parts
}

#[doc = include_str!("calc_lowering.doc/parse_full.md")]
fn parse_full(tokens: &[ParsedValue]) -> Option<LengthExpr> {
    let mut pos = 0;
    let expr = parse_sum(tokens, &mut pos)?;
    if pos == tokens.len() {
        expr.into_length()
    } else {
        None
    }
}

#[doc = include_str!("calc_lowering.doc/parse_sum.md")]
fn parse_sum(tokens: &[ParsedValue], pos: &mut usize) -> Option<Operand> {
    let mut left = parse_product(tokens, pos)?;
    while let Some(op) = peek_delim(tokens, *pos, &['+', '-']) {
        *pos += 1;
        let right = parse_product(tokens, pos)?;
        left = combine_additive(left, right, op)?;
    }
    Some(left)
}

#[doc = include_str!("calc_lowering.doc/parse_product.md")]
fn parse_product(tokens: &[ParsedValue], pos: &mut usize) -> Option<Operand> {
    let mut left = parse_unit(tokens, pos)?;
    while let Some(op) = peek_delim(tokens, *pos, &['*', '/']) {
        *pos += 1;
        let right = parse_unit(tokens, pos)?;
        left = combine_multiplicative(left, right, op)?;
    }
    Some(left)
}

#[doc = include_str!("calc_lowering.doc/parse_unit.md")]
fn parse_unit(tokens: &[ParsedValue], pos: &mut usize) -> Option<Operand> {
    let value = tokens.get(*pos)?;
    match value {
        ParsedValue::Dimension { value, unit } => {
            *pos += 1;
            dimension_length(*value, unit).map(|l| Operand::Len(LengthExpr::Leaf(l)))
        }
        ParsedValue::Percentage(p) => {
            *pos += 1;
            #[allow(clippy::cast_possible_truncation)]
            Some(Operand::Len(LengthExpr::Leaf(Length::Percent(*p as f32))))
        }
        ParsedValue::Number(n) => {
            *pos += 1;
            #[allow(clippy::cast_possible_truncation)]
            Some(Operand::Num(*n as f32))
        }
        // A nested math function or a bare parenthesised group.
        ParsedValue::Function { name, args } => {
            *pos += 1;
            lower_math_function(name, args).map(Operand::Len)
        }
        _ => None,
    }
}

#[doc = include_str!("calc_lowering.doc/peek_delim.md")]
fn peek_delim(tokens: &[ParsedValue], pos: usize, wanted: &[char]) -> Option<char> {
    match tokens.get(pos) {
        Some(ParsedValue::Delimiter(c)) if wanted.contains(c) => Some(*c),
        _ => None,
    }
}

#[doc = include_str!("calc_lowering.doc/combine_additive.md")]
fn combine_additive(left: Operand, right: Operand, op: char) -> Option<Operand> {
    match (left, right) {
        (Operand::Len(a), Operand::Len(b)) => {
            let node = if op == '+' {
                LengthExpr::Sum(Box::new(a), Box::new(b))
            } else {
                LengthExpr::Diff(Box::new(a), Box::new(b))
            };
            Some(Operand::Len(node))
        }
        (Operand::Num(a), Operand::Num(b)) => {
            Some(Operand::Num(if op == '+' { a + b } else { a - b }))
        }
        // Length ± number (or number ± length) is not a valid length.
        _ => None,
    }
}

#[doc = include_str!("calc_lowering.doc/combine_multiplicative.md")]
fn combine_multiplicative(left: Operand, right: Operand, op: char) -> Option<Operand> {
    match (left, right, op) {
        // number * number, number / number.
        (Operand::Num(a), Operand::Num(b), '*') => Some(Operand::Num(a * b)),
        (Operand::Num(a), Operand::Num(b), '/') => (b != 0.0).then_some(Operand::Num(a / b)),
        // length * number, number * length.
        (Operand::Len(a), Operand::Num(n), '*') | (Operand::Num(n), Operand::Len(a), '*') => {
            Some(Operand::Len(LengthExpr::Product(Box::new(a), n)))
        }
        // length / number only (number / length is not a length).
        (Operand::Len(a), Operand::Num(n), '/') => {
            (n != 0.0).then_some(Operand::Len(LengthExpr::Quotient(Box::new(a), n)))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::CssSource;
    use obscur_style::property::LengthContext;

    fn expr(text: &str) -> LengthExpr {
        let src = CssSource::anonymous(text);
        let values = crate::parser::ValueParser::new().parse(&src).unwrap();
        let ParsedValue::Function { name, args } = &values[0] else {
            panic!("expected a function value, got {values:?}");
        };
        lower_math_function(name, args).expect("expected a math expression")
    }

    fn ctx() -> LengthContext {
        LengthContext::new(16.0, 20.0, 800.0, 600.0)
    }

    #[test]
    fn calc_sum_mixes_px_and_em() {
        // calc(10px + 2em) -> 10 + 32 = 42.
        assert_eq!(expr("calc(10px + 2em)").resolve(&ctx()), Some(42.0));
    }

    #[test]
    fn calc_precedence_multiplication_before_addition() {
        // calc(10px + 2 * 5px) -> 10 + 10 = 20 (not (10+2)*5).
        assert_eq!(expr("calc(10px + 2 * 5px)").resolve(&ctx()), Some(20.0));
    }

    #[test]
    fn calc_number_first_multiplication() {
        assert_eq!(expr("calc(3 * 4px)").resolve(&ctx()), Some(12.0));
        assert_eq!(expr("calc(30px / 3)").resolve(&ctx()), Some(10.0));
    }

    #[test]
    fn clamp_folds_and_bounds() {
        // clamp(10px, 5vw, 100px): 5vw = 40 -> 40.
        assert_eq!(expr("clamp(10px, 5vw, 100px)").resolve(&ctx()), Some(40.0));
    }

    #[test]
    fn min_and_max_fold() {
        assert_eq!(expr("min(10px, 30px, 20px)").resolve(&ctx()), Some(10.0));
        assert_eq!(expr("max(10px, 30px, 20px)").resolve(&ctx()), Some(30.0));
    }

    #[test]
    fn nested_calc_inside_clamp() {
        // clamp(1rem, calc(1vw + 1px), 2rem): 1rem=20, calc=8+1=9, 2rem=40
        //   -> clamp -> 20 (val below min).
        assert_eq!(
            expr("clamp(1rem, calc(1vw + 1px), 2rem)").resolve(&ctx()),
            Some(20.0)
        );
    }

    #[test]
    fn parenthesised_group_overrides_precedence() {
        // calc((10px + 2px) * 3) -> 36.
        assert_eq!(expr("calc((10px + 2px) * 3)").resolve(&ctx()), Some(36.0));
    }

    #[test]
    fn length_plus_bare_number_is_rejected() {
        let src = CssSource::anonymous("calc(10px + 5)");
        let values = crate::parser::ValueParser::new().parse(&src).unwrap();
        let ParsedValue::Function { name, args } = &values[0] else {
            panic!("expected function");
        };
        assert_eq!(lower_math_function(name, args), None);
    }

    #[test]
    fn unknown_function_is_not_a_math_expression() {
        let src = CssSource::anonymous("translate(1px, 2px)");
        let values = crate::parser::ValueParser::new().parse(&src).unwrap();
        let ParsedValue::Function { name, args } = &values[0] else {
            panic!("expected function");
        };
        assert_eq!(lower_math_function(name, args), None);
    }
}
