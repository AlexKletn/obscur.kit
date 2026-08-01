#![doc = include_str!("value_parser.doc/_module.md")]

use crate::error::{CssParseError, CssParseErrorKind};
use crate::source::CssSource;

use super::css_tokenizer::{CssToken, CssTokenKind, Tokenizer};
use super::token_cursor::TokenCursor;

#[doc = include_str!("value_parser.doc/ParsedValue.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedValue {
    #[doc = include_str!("value_parser.doc/Keyword.md")]
    Keyword(Box<str>),
    #[doc = include_str!("value_parser.doc/Number.md")]
    Number(f64),
    #[doc = include_str!("value_parser.doc/Percentage.md")]
    Percentage(f64),
    #[doc = include_str!("value_parser.doc/Dimension.md")]
    Dimension {
        #[doc = include_str!("value_parser.doc/value.md")]
        value: f64,
        #[doc = include_str!("value_parser.doc/unit.md")]
        unit: Box<str>,
    },
    #[doc = include_str!("value_parser.doc/String.md")]
    String(Box<str>),
    #[doc = include_str!("value_parser.doc/Hash.md")]
    Hash(Box<str>),
    #[doc = include_str!("value_parser.doc/Function.md")]
    Function {
        #[doc = include_str!("value_parser.doc/name.md")]
        name: Box<str>,
        #[doc = include_str!("value_parser.doc/args.md")]
        args: Vec<ParsedValue>,
    },
    #[doc = include_str!("value_parser.doc/Comma.md")]
    Comma,
    #[doc = include_str!("value_parser.doc/Delimiter.md")]
    Delimiter(char),
    #[doc = include_str!("value_parser.doc/LineNames.md")]
    LineNames(Box<[Box<str>]>),
}

#[doc = include_str!("value_parser.doc/ValueParser.md")]
#[derive(Debug, Clone, Default)]
pub struct ValueParser {
    _private: (),
}

impl ValueParser {
    #[doc = include_str!("value_parser.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self { _private: () }
    }

    #[doc = include_str!("value_parser.doc/parse.md")]
    pub fn parse(&self, source: &CssSource) -> Result<Vec<ParsedValue>, CssParseError> {
        let text = source.text();
        let eof = u32::try_from(text.len()).unwrap_or(u32::MAX);
        let (tokens, mut errors) = Tokenizer::new(text).tokenize();
        if !errors.is_empty() {
            return Err(errors.remove(0));
        }
        Self::parse_tokens(&tokens, eof)
    }

    #[doc = include_str!("value_parser.doc/parse_tokens.md")]
    pub(crate) fn parse_tokens(
        tokens: &[CssToken],
        eof: u32,
    ) -> Result<Vec<ParsedValue>, CssParseError> {
        let mut cursor = TokenCursor::new(tokens, eof);
        parse_components(&mut cursor, false)
    }
}

#[doc = include_str!("value_parser.doc/parse_components.md")]
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

#[doc = include_str!("value_parser.doc/parse_component.md")]
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
        Some(CssTokenKind::LBracket) => {
            cursor.advance(); // consume '['
            parse_line_names(cursor)
        }
        _ => Err(CssParseError::new(
            CssParseErrorKind::InvalidValue,
            "unexpected token in value",
            span,
        )),
    }
}

#[doc = include_str!("value_parser.doc/parse_line_names.md")]
fn parse_line_names(cursor: &mut TokenCursor<'_>) -> Result<ParsedValue, CssParseError> {
    let mut names: Vec<Box<str>> = Vec::new();
    loop {
        cursor.skip_ws();
        match cursor.peek_kind().cloned() {
            None => return Err(CssParseError::unexpected_eof(cursor.eof())),
            Some(CssTokenKind::RBracket) => {
                cursor.advance();
                return Ok(ParsedValue::LineNames(names.into_boxed_slice()));
            }
            Some(CssTokenKind::Ident(name)) => {
                cursor.advance();
                names.push(name);
            }
            Some(_) => {
                return Err(CssParseError::new(
                    CssParseErrorKind::InvalidValue,
                    "expected a line name or ']' in a grid line-name group",
                    cursor.here_span(),
                ));
            }
        }
    }
}

#[doc = include_str!("value_parser.doc/parse_str.md")]
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
        assert_eq!(
            parse_str("flex").unwrap(),
            vec![ParsedValue::Keyword("flex".into())]
        );
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
        assert_eq!(
            parse_str("50%").unwrap(),
            vec![ParsedValue::Percentage(50.0)]
        );
    }

    #[test]
    fn parses_hash_color() {
        assert_eq!(
            parse_str("#ff0000").unwrap(),
            vec![ParsedValue::Hash("ff0000".into())]
        );
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
        assert_eq!(
            parse_str("-5px").unwrap(),
            vec![ParsedValue::Dimension {
                value: -5.0,
                unit: "px".into()
            }]
        );
    }

    #[test]
    fn empty_value_is_empty_list() {
        assert_eq!(parse_str("").unwrap(), vec![]);
    }

    #[test]
    fn parses_line_name_groups_between_tracks() {
        let v = parse_str("[sidebar-start] 200px [sidebar-end main-start] 1fr [main-end]").unwrap();
        assert_eq!(v.len(), 5);
        assert_eq!(
            v[0],
            ParsedValue::LineNames(Box::from([Box::<str>::from("sidebar-start")]))
        );
        assert_eq!(
            v[2],
            ParsedValue::LineNames(Box::from([
                Box::<str>::from("sidebar-end"),
                Box::<str>::from("main-start")
            ]))
        );
        assert_eq!(
            v[4],
            ParsedValue::LineNames(Box::from([Box::<str>::from("main-end")]))
        );
    }

    #[test]
    fn empty_line_name_group_carries_no_names() {
        assert_eq!(
            parse_str("[]").unwrap(),
            vec![ParsedValue::LineNames(Box::from([]))]
        );
    }

    #[test]
    fn unterminated_line_name_group_errors() {
        let err = parse_str("[a b").unwrap_err();
        assert_eq!(err.kind(), CssParseErrorKind::UnexpectedEof);
    }
}
