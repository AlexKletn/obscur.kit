#![doc = include_str!("../../docs/api/parser/selector_parser.doc/_module.md")]

mod selector_parse_state;

use super::{ParsedSelectorList, css_token::CssToken, css_tokenizer::Tokenizer};
use crate::error::CssParseError;
use crate::source::CssSource;
use selector_parse_state::SelectorParseState;

#[doc = include_str!("../../docs/api/parser/selector_parser.doc/SelectorParser.md")]
#[derive(Debug, Clone, Default)]
pub struct SelectorParser {
    _private: (),
}

impl SelectorParser {
    /// Creates a selector parser.
    #[must_use]
    pub fn new() -> Self {
        Self { _private: () }
    }

    #[doc = include_str!("../../docs/api/parser/selector_parser.doc/parse.md")]
    pub fn parse(&self, source: &CssSource) -> Result<ParsedSelectorList, CssParseError> {
        let text = source.text();
        let eof = u32::try_from(text.len()).unwrap_or(u32::MAX);
        let (tokens, mut errors) = Tokenizer::new(text).tokenize();
        if !errors.is_empty() {
            return Err(errors.remove(0));
        }
        Self::parse_tokens(&tokens, text, eof)
    }

    #[doc = include_str!("../../docs/api/parser/selector_parser.doc/parse_tokens.md")]
    pub(crate) fn parse_tokens(
        tokens: &[CssToken],
        source: &str,
        eof: u32,
    ) -> Result<ParsedSelectorList, CssParseError> {
        SelectorParseState::parse(tokens, source, eof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CssParseErrorKind;
    use crate::parser::{AttrOp, Combinator, ParsedComplexSelector, SimpleSelector};

    fn parse(text: &str) -> Result<ParsedSelectorList, CssParseError> {
        let src = CssSource::anonymous(text);
        SelectorParser::new().parse(&src)
    }

    fn one(text: &str) -> ParsedComplexSelector {
        parse(text).unwrap().0.into_iter().next().unwrap()
    }

    #[test]
    fn parses_type_selector() {
        let c = one("div");
        assert_eq!(c.head.parts, vec![SimpleSelector::Type("div".into())]);
        assert!(c.tail.is_empty());
    }

    #[test]
    fn parses_universal() {
        assert_eq!(one("*").head.parts, vec![SimpleSelector::Universal]);
    }

    #[test]
    fn parses_class_and_id() {
        assert_eq!(
            one(".btn").head.parts,
            vec![SimpleSelector::Class("btn".into())]
        );
        assert_eq!(
            one("#main").head.parts,
            vec![SimpleSelector::Id("main".into())]
        );
    }

    #[test]
    fn parses_compound() {
        let c = one("div.a#b");
        assert_eq!(
            c.head.parts,
            vec![
                SimpleSelector::Type("div".into()),
                SimpleSelector::Class("a".into()),
                SimpleSelector::Id("b".into()),
            ]
        );
    }

    #[test]
    fn parses_descendant_and_child() {
        let c = one("div .a");
        assert_eq!(c.tail.len(), 1);
        assert_eq!(c.tail[0].0, Combinator::Descendant);

        let c = one("div > .a");
        assert_eq!(c.tail[0].0, Combinator::Child);
    }

    #[test]
    fn parses_sibling_combinators() {
        assert_eq!(one("a + b").tail[0].0, Combinator::NextSibling);
        assert_eq!(one("a ~ b").tail[0].0, Combinator::SubsequentSibling);
    }

    #[test]
    fn parses_selector_list() {
        let list = parse("a, b, c").unwrap();
        assert_eq!(list.0.len(), 3);
    }

    #[test]
    fn parses_attribute_presence() {
        let c = one("[disabled]");
        assert_eq!(
            c.head.parts,
            vec![SimpleSelector::Attribute {
                name: "disabled".into(),
                op: None,
                value: None
            }]
        );
    }

    #[test]
    fn parses_attribute_operators() {
        let cases = [
            ("[a=b]", AttrOp::Exact),
            ("[a~=b]", AttrOp::Includes),
            ("[a|=b]", AttrOp::DashMatch),
            ("[a^=b]", AttrOp::Prefix),
            ("[a$=b]", AttrOp::Suffix),
            ("[a*=b]", AttrOp::Substring),
        ];
        for (text, expected) in cases {
            let c = one(text);
            let SimpleSelector::Attribute { op, .. } = &c.head.parts[0] else {
                panic!("expected attribute selector for {text}");
            };
            assert_eq!(*op, Some(expected), "for {text}");
        }
    }

    #[test]
    fn parses_attribute_quoted_value() {
        let c = one("[type=\"text\"]");
        let SimpleSelector::Attribute { value, .. } = &c.head.parts[0] else {
            panic!("expected attribute selector");
        };
        assert_eq!(value.as_deref(), Some("text"));
    }

    #[test]
    fn parses_pseudo_class() {
        let c = one(":hover");
        assert_eq!(
            c.head.parts,
            vec![SimpleSelector::PseudoClass {
                name: "hover".into(),
                arg: None
            }]
        );
    }

    #[test]
    fn parses_functional_pseudo_class() {
        let c = one(":nth-child(2n + 1)");
        let SimpleSelector::PseudoClass { name, arg } = &c.head.parts[0] else {
            panic!("expected pseudo-class");
        };
        assert_eq!(&**name, "nth-child");
        assert_eq!(arg.as_deref(), Some("2n + 1"));
    }

    #[test]
    fn parses_pseudo_element() {
        let c = one("::before");
        assert_eq!(
            c.head.parts,
            vec![SimpleSelector::PseudoElement("before".into())]
        );
    }

    #[test]
    fn complex_with_pseudo_and_combinator() {
        let c = one("ul > li:hover");
        assert_eq!(c.head.parts, vec![SimpleSelector::Type("ul".into())]);
        assert_eq!(c.tail.len(), 1);
        assert_eq!(c.tail[0].0, Combinator::Child);
        assert_eq!(
            c.tail[0].1.parts,
            vec![
                SimpleSelector::Type("li".into()),
                SimpleSelector::PseudoClass {
                    name: "hover".into(),
                    arg: None
                },
            ]
        );
    }

    #[test]
    fn empty_selector_is_error() {
        assert_eq!(
            parse("").unwrap_err().kind(),
            CssParseErrorKind::InvalidSelector
        );
    }

    #[test]
    fn dangling_class_is_error() {
        assert_eq!(
            parse(".").unwrap_err().kind(),
            CssParseErrorKind::InvalidSelector
        );
    }

    #[test]
    fn unterminated_attribute_is_error() {
        assert_eq!(
            parse("[a").unwrap_err().kind(),
            CssParseErrorKind::InvalidSelector
        );
    }
}
