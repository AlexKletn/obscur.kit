//! The [`ValueParser`] and its parsed component-value AST.
//!
//! A declaration value is modelled as the CSS *component value* list (spec
//! §4.13): a flat [`Vec<ParsedValue>`] of space-separated components in which
//! commas and operators survive as their own [`ParsedValue::Comma`] /
//! [`ParsedValue::Delimiter`] components. Functions nest their arguments as a
//! child component list. Nothing is evaluated here — `calc()` and friends keep
//! their raw operands so the compiler can lower or fold them later.

use crate::error::{CssParseError, CssParseErrorKind};
use crate::source::CssSource;

use super::stylesheet_parser::{CssToken, CssTokenKind, TokenCursor, Tokenizer};

/// A single parsed CSS component value (spec §4.13).
///
/// This is a *syntactic* model: no unit validation, colour resolution, or
/// arithmetic happens here. Commas and bare operator characters are preserved as
/// [`Comma`](ParsedValue::Comma) and [`Delimiter`](ParsedValue::Delimiter)
/// components so a value round-trips structurally into the compiler.
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedValue {
    /// A bare identifier keyword such as `flex`, `auto`, or `inherit`.
    Keyword(Box<str>),
    /// A unitless number such as `12` or `-0.5`.
    Number(f64),
    /// A percentage such as `50%`; stores the numeric part (`50`).
    Percentage(f64),
    /// A dimension such as `12px` or `1.5rem`; the unit is kept verbatim and is
    /// validated later by the compiler.
    Dimension {
        /// The numeric part of the dimension.
        value: f64,
        /// The unit text (for example `px`, `rem`, `fr`, `deg`, `ms`).
        unit: Box<str>,
    },
    /// A quoted string (without its quotes).
    String(Box<str>),
    /// A hash value such as a hex colour `#ff0000`; stored without the `#`.
    Hash(Box<str>),
    /// A function call such as `calc(…)`, `rgb(…)`, `var(…)`, or `translate(…)`.
    ///
    /// A bare parenthesised group `( … )` is represented as a function with an
    /// empty [`name`](ParsedValue::Function::name). Arguments are the raw nested
    /// component list, commas and operators included.
    Function {
        /// The function name (empty for a bare parenthesised group).
        name: Box<str>,
        /// The nested argument component list.
        args: Vec<ParsedValue>,
    },
    /// A `,` comma separator between components (for example between font
    /// families or `transition` layers).
    Comma,
    /// A bare operator/delimiter character (for example `+`, `-`, `*`, `/`
    /// inside `calc()`).
    Delimiter(char),
}

/// Parses a declaration's value tokens into a component-value list.
///
/// The parser is stateless; a single instance may parse any number of values.
#[derive(Debug, Clone, Default)]
pub struct ValueParser {
    _private: (),
}

impl ValueParser {
    /// Creates a value parser.
    #[must_use]
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Parses `source` as a standalone component-value list.
    ///
    /// # Errors
    ///
    /// Returns a [`CssParseError`] if a function or parenthesised group is
    /// unterminated, or a stray closing delimiter or block token appears where a
    /// value was expected.
    pub fn parse(&self, source: &CssSource) -> Result<Vec<ParsedValue>, CssParseError> {
        let text = source.text();
        let eof = u32::try_from(text.len()).unwrap_or(u32::MAX);
        let (tokens, mut errors) = Tokenizer::new(text).tokenize();
        if !errors.is_empty() {
            return Err(errors.remove(0));
        }
        Self::parse_tokens(&tokens, eof)
    }

    /// Parses a component-value list from an already-tokenized slice.
    ///
    /// # Errors
    ///
    /// Returns a [`CssParseError`] on an unterminated function/group or a stray
    /// closing/block token.
    pub(crate) fn parse_tokens(
        tokens: &[CssToken],
        eof: u32,
    ) -> Result<Vec<ParsedValue>, CssParseError> {
        let mut cursor = TokenCursor::new(tokens, eof);
        parse_components(&mut cursor, false)
    }
}

/// Parses a run of components. When `in_function`, stops at (and consumes) the
/// matching `)`; otherwise stops at end-of-input.
fn parse_components(
    cursor: &mut TokenCursor<'_>,
    in_function: bool,
) -> Result<Vec<ParsedValue>, CssParseError> {
    let mut out = Vec::new();
    loop {
        cursor.skip_ws();
        match cursor.peek_kind() {
            None => {
                if in_function {
                    return Err(CssParseError::unexpected_eof(cursor.eof()));
                }
                return Ok(out);
            }
            Some(CssTokenKind::RParen) => {
                if in_function {
                    cursor.advance();
                    return Ok(out);
                }
                return Err(CssParseError::new(
                    CssParseErrorKind::InvalidValue,
                    "unexpected ')'",
                    cursor.here_span(),
                ));
            }
            Some(_) => out.push(parse_component(cursor)?),
        }
    }
}

/// Parses exactly one component value.
fn parse_component(cursor: &mut TokenCursor<'_>) -> Result<ParsedValue, CssParseError> {
    let span = cursor.here_span();
    let kind = cursor.peek_kind().cloned();
    match kind {
        Some(CssTokenKind::Ident(name)) => {
            cursor.advance();
            // An ident immediately followed by `(` (no whitespace, since
            // whitespace is its own token) is a function.
            if matches!(cursor.peek_kind(), Some(CssTokenKind::LParen)) {
                cursor.advance(); // consume '('
                let args = parse_components(cursor, true)?;
                Ok(ParsedValue::Function { name, args })
            } else {
                Ok(ParsedValue::Keyword(name))
            }
        }
        Some(CssTokenKind::Number(v)) => {
            cursor.advance();
            Ok(ParsedValue::Number(v))
        }
        Some(CssTokenKind::Percentage(v)) => {
            cursor.advance();
            Ok(ParsedValue::Percentage(v))
        }
        Some(CssTokenKind::Dimension { value, unit }) => {
            cursor.advance();
            Ok(ParsedValue::Dimension { value, unit })
        }
        Some(CssTokenKind::Str(s)) => {
            cursor.advance();
            Ok(ParsedValue::String(s))
        }
        Some(CssTokenKind::Hash(h)) => {
            cursor.advance();
            Ok(ParsedValue::Hash(h))
        }
        Some(CssTokenKind::Comma) => {
            cursor.advance();
            Ok(ParsedValue::Comma)
        }
        Some(CssTokenKind::LParen) => {
            cursor.advance();
            let args = parse_components(cursor, true)?;
            Ok(ParsedValue::Function {
                name: "".into(),
                args,
            })
        }
        Some(CssTokenKind::Delim(c)) => {
            cursor.advance();
            Ok(ParsedValue::Delimiter(c))
        }
        _ => Err(CssParseError::new(
            CssParseErrorKind::InvalidValue,
            "unexpected token in value",
            span,
        )),
    }
}

/// Tokenizes `text` and parses it as a component-value list (test helper).
#[cfg(test)]
fn parse_str(text: &str) -> Result<Vec<ParsedValue>, CssParseError> {
    let src = CssSource::anonymous(text);
    ValueParser::new().parse(&src)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_keyword() {
        assert_eq!(parse_str("flex").unwrap(), vec![ParsedValue::Keyword("flex".into())]);
    }

    #[test]
    fn parses_number_and_dimension() {
        assert_eq!(parse_str("12").unwrap(), vec![ParsedValue::Number(12.0)]);
        assert_eq!(
            parse_str("12px").unwrap(),
            vec![ParsedValue::Dimension {
                value: 12.0,
                unit: "px".into()
            }]
        );
    }

    #[test]
    fn parses_percentage() {
        assert_eq!(parse_str("50%").unwrap(), vec![ParsedValue::Percentage(50.0)]);
    }

    #[test]
    fn parses_hash_color() {
        assert_eq!(parse_str("#ff0000").unwrap(), vec![ParsedValue::Hash("ff0000".into())]);
    }

    #[test]
    fn parses_string() {
        assert_eq!(
            parse_str("\"Arial\"").unwrap(),
            vec![ParsedValue::String("Arial".into())]
        );
    }

    #[test]
    fn parses_space_separated_list() {
        assert_eq!(
            parse_str("1px 2px 3px").unwrap(),
            vec![
                ParsedValue::Dimension {
                    value: 1.0,
                    unit: "px".into()
                },
                ParsedValue::Dimension {
                    value: 2.0,
                    unit: "px".into()
                },
                ParsedValue::Dimension {
                    value: 3.0,
                    unit: "px".into()
                },
            ]
        );
    }

    #[test]
    fn parses_comma_list() {
        let v = parse_str("Arial, sans-serif").unwrap();
        assert_eq!(
            v,
            vec![
                ParsedValue::Keyword("Arial".into()),
                ParsedValue::Comma,
                ParsedValue::Keyword("sans-serif".into()),
            ]
        );
    }

    #[test]
    fn parses_rgb_function() {
        let v = parse_str("rgb(255, 0, 0)").unwrap();
        assert_eq!(
            v,
            vec![ParsedValue::Function {
                name: "rgb".into(),
                args: vec![
                    ParsedValue::Number(255.0),
                    ParsedValue::Comma,
                    ParsedValue::Number(0.0),
                    ParsedValue::Comma,
                    ParsedValue::Number(0.0),
                ]
            }]
        );
    }

    #[test]
    fn parses_calc_keeps_operators() {
        let v = parse_str("calc(100% - 20px)").unwrap();
        assert_eq!(
            v,
            vec![ParsedValue::Function {
                name: "calc".into(),
                args: vec![
                    ParsedValue::Percentage(100.0),
                    ParsedValue::Delimiter('-'),
                    ParsedValue::Dimension {
                        value: 20.0,
                        unit: "px".into()
                    },
                ]
            }]
        );
    }

    #[test]
    fn parses_nested_functions() {
        let v = parse_str("clamp(1rem, calc(1vw + 1px), 2rem)").unwrap();
        let ParsedValue::Function { name, args } = &v[0] else {
            panic!("expected function");
        };
        assert_eq!(&**name, "clamp");
        // args: 1rem , <calc> , 2rem
        assert!(matches!(args[0], ParsedValue::Dimension { .. }));
        assert_eq!(args[1], ParsedValue::Comma);
        assert!(matches!(&args[2], ParsedValue::Function { name, .. } if &**name == "calc"));
    }

    #[test]
    fn parses_var_function() {
        let v = parse_str("var(--main-color)").unwrap();
        assert_eq!(
            v,
            vec![ParsedValue::Function {
                name: "var".into(),
                args: vec![ParsedValue::Keyword("--main-color".into())]
            }]
        );
    }

    #[test]
    fn parses_bare_paren_group() {
        let v = parse_str("(1px + 2px)").unwrap();
        let ParsedValue::Function { name, args } = &v[0] else {
            panic!("expected group");
        };
        assert_eq!(&**name, "");
        assert_eq!(args.len(), 3);
    }

    #[test]
    fn unterminated_function_errors() {
        let err = parse_str("calc(1px + ").unwrap_err();
        assert_eq!(err.kind(), CssParseErrorKind::UnexpectedEof);
    }

    #[test]
    fn stray_close_paren_errors() {
        let err = parse_str("1px )").unwrap_err();
        assert_eq!(err.kind(), CssParseErrorKind::InvalidValue);
    }

    #[test]
    fn negative_number_is_one_component() {
        assert_eq!(parse_str("-5px").unwrap(), vec![ParsedValue::Dimension {
            value: -5.0,
            unit: "px".into()
        }]);
    }

    #[test]
    fn empty_value_is_empty_list() {
        assert_eq!(parse_str("").unwrap(), vec![]);
    }
}
