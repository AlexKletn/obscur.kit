#![doc = include_str!("../../docs/api/compiler/calc_lowering.doc/_module.md")]

mod clamp_bound;
mod length_operand;

use clamp_bound::{ClampBound, parse_clamp_bound};

use length_operand::LengthOperand;

use super::value_lowering::dimension_length;
use crate::parser::ParsedValue;

use obscur_style::matching::StyleRangeValue;
use obscur_style::property::{Length, LengthExpr};

#[doc = include_str!("../../docs/api/compiler/calc_lowering.doc/lower_math_function.md")]
pub(crate) fn lower_math_function(name: &str, args: &[ParsedValue]) -> Option<LengthExpr> {
    lower_operand(name, args)?.into_length()
}

pub(super) fn lower_range_math(name: &str, args: &[ParsedValue]) -> Option<StyleRangeValue> {
    match lower_operand(name, args)? {
        LengthOperand::Len(expr) => Some(StyleRangeValue::Math(expr.into())),
        LengthOperand::Num(value) if value.is_finite() => Some(StyleRangeValue::Number(value)),
        LengthOperand::Num(_) => None,
    }
}

fn lower_operand(name: &str, args: &[ParsedValue]) -> Option<LengthOperand> {
    if name.eq_ignore_ascii_case("calc") || name.is_empty() {
        parse_full(args)
    } else if name.eq_ignore_ascii_case("min") {
        lower_extrema(args, true)
    } else if name.eq_ignore_ascii_case("max") {
        lower_extrema(args, false)
    } else if name.eq_ignore_ascii_case("clamp") {
        let parts = split_top_level_commas(args);
        let [first, center, last] = parts.as_slice() else {
            return None;
        };
        lower_clamp(
            parse_clamp_bound(first)?,
            parse_full(center)?,
            parse_clamp_bound(last)?,
        )
    } else {
        None
    }
}

fn lower_extrema(args: &[ParsedValue], is_min: bool) -> Option<LengthOperand> {
    let parts = split_top_level_commas(args);
    let mut operands = parts.into_iter().map(parse_full);
    match operands.next()?? {
        LengthOperand::Len(first) => {
            let mut lengths = Vec::with_capacity(operands.len() + 1);
            lengths.push(first);
            for operand in operands {
                lengths.push(operand?.into_length()?);
            }
            Some(LengthOperand::Len(if is_min {
                LengthExpr::Min(lengths)
            } else {
                LengthExpr::Max(lengths)
            }))
        }
        LengthOperand::Num(first) => {
            let combine = if is_min { f32::min } else { f32::max };
            let result = operands.try_fold(first, |acc, operand| match operand? {
                LengthOperand::Num(value) => Some(combine(acc, value)),
                LengthOperand::Len(_) => None,
            })?;
            Some(LengthOperand::Num(result))
        }
    }
}

fn lower_clamp(min: ClampBound, value: LengthOperand, max: ClampBound) -> Option<LengthOperand> {
    use ClampBound::{Bounded, None as Unbounded};
    use LengthOperand::{Len, Num};

    Some(match (min, value, max) {
        (Bounded(Len(min)), Len(val), Bounded(Len(max))) => Len(LengthExpr::Clamp(
            Box::new(min),
            Box::new(val),
            Box::new(max),
        )),
        (Bounded(Len(min)), Len(val), Unbounded) => Len(LengthExpr::Max(vec![min, val])),
        (Unbounded, Len(val), Bounded(Len(max))) => Len(LengthExpr::Min(vec![val, max])),
        (Bounded(Num(min)), Num(val), Bounded(Num(max))) => Num(min.max(val.min(max))),
        (Bounded(Num(min)), Num(val), Unbounded) => Num(min.max(val)),
        (Unbounded, Num(val), Bounded(Num(max))) => Num(val.min(max)),
        (Unbounded, val, Unbounded) => val,
        _ => return None,
    })
}

/// Splits `args` at top-level [`ParsedValue::Comma`]s (nested function arguments
/// keep their own commas inside their [`ParsedValue::Function`] node, so they are
/// never seen here).
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

/// Parses one complete arithmetic operand, preserving whether it is a number or a length expression.
fn parse_full(tokens: &[ParsedValue]) -> Option<LengthOperand> {
    let mut pos = 0;
    let expr = parse_sum(tokens, &mut pos)?;
    if pos == tokens.len() {
        Some(expr)
    } else {
        None
    }
}

/// `sum := product (('+' | '-') product)*`
fn parse_sum(tokens: &[ParsedValue], pos: &mut usize) -> Option<LengthOperand> {
    let mut left = parse_product(tokens, pos)?;
    while let Some(op) = peek_delim(tokens, *pos, &['+', '-']) {
        *pos += 1;
        let right = parse_product(tokens, pos)?;
        left = combine_additive(left, right, op)?;
    }
    Some(left)
}

/// `product := unit (('*' | '/') unit)*`
fn parse_product(tokens: &[ParsedValue], pos: &mut usize) -> Option<LengthOperand> {
    let mut left = parse_unit(tokens, pos)?;
    while let Some(op) = peek_delim(tokens, *pos, &['*', '/']) {
        *pos += 1;
        let right = parse_unit(tokens, pos)?;
        left = combine_multiplicative(left, right, op)?;
    }
    Some(left)
}

/// `unit := <length> | <number> | ( sum ) | <math-fn>`
fn parse_unit(tokens: &[ParsedValue], pos: &mut usize) -> Option<LengthOperand> {
    let value = tokens.get(*pos)?;
    match value {
        ParsedValue::Dimension { value, unit } => {
            *pos += 1;
            dimension_length(*value, unit).map(|l| LengthOperand::Len(LengthExpr::Leaf(l)))
        }
        ParsedValue::Percentage(p) => {
            *pos += 1;
            #[allow(clippy::cast_possible_truncation)]
            Some(LengthOperand::Len(LengthExpr::Leaf(Length::Percent(
                *p as f32,
            ))))
        }
        ParsedValue::Number(n) => {
            *pos += 1;
            #[allow(clippy::cast_possible_truncation)]
            Some(LengthOperand::Num(*n as f32))
        }
        // A nested math function or a bare parenthesised group.
        ParsedValue::Function { name, args } => {
            *pos += 1;
            lower_operand(name, args)
        }
        _ => None,
    }
}

/// Returns the delimiter char at `pos` if it is one of `wanted`.
fn peek_delim(tokens: &[ParsedValue], pos: usize, wanted: &[char]) -> Option<char> {
    match tokens.get(pos) {
        Some(ParsedValue::Delimiter(c)) if wanted.contains(c) => Some(*c),
        _ => None,
    }
}

/// Combines `left op right` for `+` / `-`. Both operands must be lengths (a
/// length plus a bare number is invalid), except two numbers fold to a number.
fn combine_additive(left: LengthOperand, right: LengthOperand, op: char) -> Option<LengthOperand> {
    match (left, right) {
        (LengthOperand::Len(a), LengthOperand::Len(b)) => {
            let node = if op == '+' {
                LengthExpr::Sum(Box::new(a), Box::new(b))
            } else {
                LengthExpr::Diff(Box::new(a), Box::new(b))
            };
            Some(LengthOperand::Len(node))
        }
        (LengthOperand::Num(a), LengthOperand::Num(b)) => {
            Some(LengthOperand::Num(if op == '+' { a + b } else { a - b }))
        }
        // Length ± number (or number ± length) is not a valid length.
        _ => None,
    }
}

/// Combines `left op right` for `*` / `/`. Exactly one operand must be a number
/// (division additionally requires the *right* operand to be the number).
fn combine_multiplicative(
    left: LengthOperand,
    right: LengthOperand,
    op: char,
) -> Option<LengthOperand> {
    match (left, right, op) {
        // number * number, number / number.
        (LengthOperand::Num(a), LengthOperand::Num(b), '*') => Some(LengthOperand::Num(a * b)),
        (LengthOperand::Num(a), LengthOperand::Num(b), '/') => {
            (b != 0.0).then_some(LengthOperand::Num(a / b))
        }
        // length * number, number * length.
        (LengthOperand::Len(a), LengthOperand::Num(n), '*')
        | (LengthOperand::Num(n), LengthOperand::Len(a), '*') => {
            Some(LengthOperand::Len(LengthExpr::Product(Box::new(a), n)))
        }
        // length / number only (number / length is not a length).
        (LengthOperand::Len(a), LengthOperand::Num(n), '/') => {
            (n != 0.0).then_some(LengthOperand::Len(LengthExpr::Quotient(Box::new(a), n)))
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
