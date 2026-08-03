#![doc = include_str!("stylesheet_parser.doc/_module.md")]

use crate::error::{
    CssParseError, CssParseErrorKind, UnsupportedCssFeature, UnsupportedFeatureKind,
};
use crate::source::{CssSource, SourceSpan};

use super::container_query_parser::{ContainerQueryParser, ParsedContainerQuery};
use super::css_tokenizer::{CssToken, CssTokenKind, Tokenizer};
use super::media_query_parser::{MediaQueryParser, ParsedMediaQueryList};
use super::property_parser::{ParsedDeclaration, PropertyParser};
use super::selector_parser::{ParsedSelectorList, SelectorParser};

#[doc = include_str!("stylesheet_parser.doc/ParsedStylesheet.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedStylesheet {
    #[doc = include_str!("stylesheet_parser.doc/rules.md")]
    pub rules: Vec<ParsedRule>,
    #[doc = include_str!("stylesheet_parser.doc/errors.md")]
    pub errors: Vec<CssParseError>,
    #[doc = include_str!("stylesheet_parser.doc/unsupported.md")]
    pub unsupported: Vec<UnsupportedCssFeature>,
}

#[doc = include_str!("stylesheet_parser.doc/ParsedRule.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedRule {
    #[doc = include_str!("stylesheet_parser.doc/Style.md")]
    Style(ParsedStyleRule),
    #[doc = include_str!("stylesheet_parser.doc/Media.md")]
    Media(ParsedMediaRule),
    #[doc = include_str!("stylesheet_parser.doc/Container.md")]
    Container(ParsedContainerRule),
    #[doc = include_str!("stylesheet_parser.doc/Keyframes.md")]
    Keyframes(ParsedKeyframesRule),
}

#[doc = include_str!("stylesheet_parser.doc/ParsedStyleRule.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedStyleRule {
    #[doc = include_str!("stylesheet_parser.doc/selectors.md")]
    pub selectors: ParsedSelectorList,
    #[doc = include_str!("stylesheet_parser.doc/declarations.md")]
    pub declarations: Vec<ParsedDeclaration>,
}

#[doc = include_str!("stylesheet_parser.doc/ParsedMediaRule.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedMediaRule {
    #[doc = include_str!("stylesheet_parser.doc/query.md")]
    pub query: ParsedMediaQueryList,
    #[doc = include_str!("stylesheet_parser.doc/rules_2.md")]
    pub rules: Vec<ParsedRule>,
}

#[doc = include_str!("stylesheet_parser.doc/ParsedContainerRule.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedContainerRule {
    #[doc = include_str!("stylesheet_parser.doc/query_2.md")]
    pub query: ParsedContainerQuery,
    #[doc = include_str!("stylesheet_parser.doc/rules_3.md")]
    pub rules: Vec<ParsedRule>,
}

#[doc = include_str!("stylesheet_parser.doc/ParsedKeyframesRule.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedKeyframesRule {
    #[doc = include_str!("stylesheet_parser.doc/name.md")]
    pub name: Box<str>,
    #[doc = include_str!("stylesheet_parser.doc/keyframes.md")]
    pub keyframes: Vec<ParsedKeyframe>,
}

#[doc = include_str!("stylesheet_parser.doc/ParsedKeyframe.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedKeyframe {
    #[doc = include_str!("stylesheet_parser.doc/selectors_2.md")]
    pub selectors: Vec<KeyframeSelector>,
    #[doc = include_str!("stylesheet_parser.doc/declarations_2.md")]
    pub declarations: Vec<ParsedDeclaration>,
}

#[doc = include_str!("stylesheet_parser.doc/KeyframeSelector.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyframeSelector {
    #[doc = include_str!("stylesheet_parser.doc/From.md")]
    From,
    #[doc = include_str!("stylesheet_parser.doc/To.md")]
    To,
    #[doc = include_str!("stylesheet_parser.doc/Percent.md")]
    Percent(f64),
}

#[doc = include_str!("stylesheet_parser.doc/StylesheetParser.md")]
#[derive(Debug)]
pub struct StylesheetParser<'a> {
    source: &'a CssSource,
}

impl<'a> StylesheetParser<'a> {
    #[doc = include_str!("stylesheet_parser.doc/new.md")]
    #[must_use]
    pub fn new(source: &'a CssSource) -> Self {
        Self { source }
    }

    #[doc = include_str!("stylesheet_parser.doc/parse.md")]
    #[must_use]
    pub fn parse(&self) -> ParsedStylesheet {
        let text = self.source.text();
        let eof = u32::try_from(text.len()).unwrap_or(u32::MAX);
        let (tokens, tok_errors) = Tokenizer::new(text).tokenize();
        let mut builder = Builder {
            text,
            tokens,
            pos: 0,
            eof,
            errors: tok_errors,
            unsupported: Vec::new(),
        };
        let rules = builder.parse_rules(false);
        ParsedStylesheet {
            rules,
            errors: builder.errors,
            unsupported: builder.unsupported,
        }
    }
}

#[doc = include_str!("stylesheet_parser.doc/Builder.md")]
struct Builder<'a> {
    text: &'a str,
    tokens: Vec<CssToken>,
    pos: usize,
    eof: u32,
    errors: Vec<CssParseError>,
    unsupported: Vec<UnsupportedCssFeature>,
}

impl Builder<'_> {
    fn cur_kind(&self) -> Option<&CssTokenKind> {
        self.tokens.get(self.pos).map(CssToken::kind)
    }

    fn cur_span(&self) -> SourceSpan {
        self.tokens
            .get(self.pos)
            .map_or_else(|| SourceSpan::empty(self.eof), CssToken::span)
    }

    fn skip_ws(&mut self) {
        while matches!(self.tokens.get(self.pos), Some(t) if t.is_whitespace()) {
            self.pos += 1;
        }
    }

    #[doc = include_str!("stylesheet_parser.doc/parse_rules.md")]
    fn parse_rules(&mut self, nested: bool) -> Vec<ParsedRule> {
        let mut rules = Vec::new();
        loop {
            self.skip_ws();
            match self.cur_kind() {
                None => {
                    if nested {
                        self.errors.push(CssParseError::new(
                            CssParseErrorKind::UnterminatedBlock,
                            "unterminated block",
                            SourceSpan::empty(self.eof),
                        ));
                    }
                    break;
                }
                Some(CssTokenKind::RBrace) if nested => {
                    self.pos += 1;
                    break;
                }
                Some(CssTokenKind::AtKeyword(_)) => {
                    if let Some(rule) = self.parse_at_rule() {
                        rules.push(rule);
                    }
                }
                Some(_) => {
                    if let Some(rule) = self.parse_style_rule() {
                        rules.push(rule);
                    }
                }
            }
        }
        rules
    }

    #[doc = include_str!("stylesheet_parser.doc/parse_style_rule.md")]
    fn parse_style_rule(&mut self) -> Option<ParsedRule> {
        let start = self.pos;
        let Some(lbrace) = self.find_lbrace() else {
            self.errors.push(CssParseError::new(
                CssParseErrorKind::UnterminatedBlock,
                "expected '{' to open a declaration block",
                self.cur_span(),
            ));
            self.recover();
            return None;
        };
        let prelude = &self.tokens[start..lbrace];
        let selectors = SelectorParser::parse_tokens(prelude, self.text, self.eof);
        self.pos = lbrace + 1; // consume '{'
        match selectors {
            Ok(selectors) => {
                let declarations = self.parse_declaration_block();
                Some(ParsedRule::Style(ParsedStyleRule {
                    selectors,
                    declarations,
                }))
            }
            Err(err) => {
                self.errors.push(err);
                self.skip_block_body();
                None
            }
        }
    }

    #[doc = include_str!("stylesheet_parser.doc/parse_at_rule.md")]
    fn parse_at_rule(&mut self) -> Option<ParsedRule> {
        let at_span = self.cur_span();
        let Some(CssTokenKind::AtKeyword(name)) = self.cur_kind() else {
            return None;
        };
        let name = name.to_ascii_lowercase();
        self.pos += 1; // consume the at-keyword

        match name.as_str() {
            "media" => self.parse_media_rule(),
            "container" => {
                // The query parses and lowers, but nothing can establish a query
                // container — there is no `container-type` property, and a
                // container's size is a layout output that whole-tree style
                // matching runs before. The condition therefore fails closed and
                // the block's rules never apply, so say so (§4.13) instead of
                // letting the author believe their rules are live.
                self.unsupported.push(UnsupportedCssFeature::new(
                    UnsupportedFeatureKind::AtRule,
                    "@container",
                    at_span,
                ));
                self.parse_container_rule()
            }
            "keyframes" | "-webkit-keyframes" => self.parse_keyframes_rule(),
            _ => {
                self.unsupported.push(UnsupportedCssFeature::new(
                    UnsupportedFeatureKind::AtRule,
                    format!("@{name}"),
                    at_span,
                ));
                self.skip_at_rule();
                None
            }
        }
    }

    #[doc = include_str!("stylesheet_parser.doc/parse_media_rule.md")]
    fn parse_media_rule(&mut self) -> Option<ParsedRule> {
        let start = self.pos;
        let Some(lbrace) = self.find_lbrace() else {
            self.errors.push(CssParseError::new(
                CssParseErrorKind::InvalidAtRule,
                "expected '{' after @media prelude",
                self.cur_span(),
            ));
            self.recover();
            return None;
        };
        let prelude = &self.tokens[start..lbrace];
        let query = match MediaQueryParser::parse_tokens(prelude, self.eof) {
            Ok(query) => query,
            Err(err) => {
                self.errors.push(err);
                ParsedMediaQueryList::default()
            }
        };
        self.pos = lbrace + 1;
        let rules = self.parse_rules(true);
        Some(ParsedRule::Media(ParsedMediaRule { query, rules }))
    }

    #[doc = include_str!("stylesheet_parser.doc/parse_container_rule.md")]
    fn parse_container_rule(&mut self) -> Option<ParsedRule> {
        let start = self.pos;
        let Some(lbrace) = self.find_lbrace() else {
            self.errors.push(CssParseError::new(
                CssParseErrorKind::InvalidAtRule,
                "expected '{' after @container prelude",
                self.cur_span(),
            ));
            self.recover();
            return None;
        };
        let prelude = &self.tokens[start..lbrace];
        let query = match ContainerQueryParser::parse_tokens(prelude, self.eof) {
            Ok(query) => query,
            Err(err) => {
                self.errors.push(err);
                ParsedContainerQuery::default()
            }
        };
        self.pos = lbrace + 1;
        let rules = self.parse_rules(true);
        Some(ParsedRule::Container(ParsedContainerRule { query, rules }))
    }

    #[doc = include_str!("stylesheet_parser.doc/parse_keyframes_rule.md")]
    fn parse_keyframes_rule(&mut self) -> Option<ParsedRule> {
        self.skip_ws();
        let name: Box<str> =
            if let Some(CssTokenKind::Ident(n) | CssTokenKind::Str(n)) = self.cur_kind() {
                let n = n.clone();
                self.pos += 1;
                n
            } else {
                self.errors.push(CssParseError::new(
                    CssParseErrorKind::InvalidAtRule,
                    "expected a name after @keyframes",
                    self.cur_span(),
                ));
                Box::<str>::from("")
            };
        self.skip_ws();
        if !matches!(self.cur_kind(), Some(CssTokenKind::LBrace)) {
            self.errors.push(CssParseError::new(
                CssParseErrorKind::InvalidAtRule,
                "expected '{' after @keyframes name",
                self.cur_span(),
            ));
            self.recover();
            return None;
        }
        self.pos += 1; // consume '{'

        let mut keyframes = Vec::new();
        loop {
            self.skip_ws();
            match self.cur_kind() {
                None => {
                    self.errors.push(CssParseError::new(
                        CssParseErrorKind::UnterminatedBlock,
                        "unterminated @keyframes block",
                        SourceSpan::empty(self.eof),
                    ));
                    break;
                }
                Some(CssTokenKind::RBrace) => {
                    self.pos += 1;
                    break;
                }
                Some(_) => {
                    if let Some(frame) = self.parse_keyframe() {
                        keyframes.push(frame);
                    }
                }
            }
        }
        Some(ParsedRule::Keyframes(ParsedKeyframesRule {
            name,
            keyframes,
        }))
    }

    #[doc = include_str!("stylesheet_parser.doc/parse_keyframe.md")]
    fn parse_keyframe(&mut self) -> Option<ParsedKeyframe> {
        let start = self.pos;
        let Some(lbrace) = self.find_lbrace() else {
            self.errors.push(CssParseError::new(
                CssParseErrorKind::UnterminatedBlock,
                "expected '{' in @keyframes block",
                self.cur_span(),
            ));
            self.recover();
            return None;
        };
        let selectors = self.parse_keyframe_selectors(start, lbrace);
        self.pos = lbrace + 1;
        let declarations = self.parse_declaration_block();
        Some(ParsedKeyframe {
            selectors,
            declarations,
        })
    }

    #[doc = include_str!("stylesheet_parser.doc/parse_keyframe_selectors.md")]
    fn parse_keyframe_selectors(&mut self, start: usize, lbrace: usize) -> Vec<KeyframeSelector> {
        let mut selectors = Vec::new();
        let mut errors = Vec::new();
        for tok in &self.tokens[start..lbrace] {
            match tok.kind() {
                CssTokenKind::Ident(id) if id.eq_ignore_ascii_case("from") => {
                    selectors.push(KeyframeSelector::From);
                }
                CssTokenKind::Ident(id) if id.eq_ignore_ascii_case("to") => {
                    selectors.push(KeyframeSelector::To);
                }
                CssTokenKind::Percentage(v) | CssTokenKind::Number(v) => {
                    selectors.push(KeyframeSelector::Percent(*v));
                }
                CssTokenKind::Whitespace | CssTokenKind::Comma => {}
                _ => errors.push(CssParseError::new(
                    CssParseErrorKind::InvalidSelector,
                    "invalid keyframe selector",
                    tok.span(),
                )),
            }
        }
        self.errors.append(&mut errors);
        selectors
    }

    #[doc = include_str!("stylesheet_parser.doc/parse_declaration_block.md")]
    fn parse_declaration_block(&mut self) -> Vec<ParsedDeclaration> {
        let mut declarations = Vec::new();
        loop {
            self.skip_ws();
            match self.cur_kind() {
                None => {
                    self.errors.push(CssParseError::new(
                        CssParseErrorKind::UnterminatedBlock,
                        "unterminated declaration block",
                        SourceSpan::empty(self.eof),
                    ));
                    break;
                }
                Some(CssTokenKind::RBrace) => {
                    self.pos += 1;
                    break;
                }
                Some(CssTokenKind::Semicolon) => {
                    self.pos += 1; // tolerate empty declarations
                }
                Some(_) => {
                    let (decl_start, terminator) = self.collect_declaration();
                    let slice = trim_semicolon(&self.tokens[decl_start..self.pos]);
                    if !slice.is_empty() {
                        match PropertyParser::parse_tokens(slice, self.eof) {
                            Ok(decl) => declarations.push(decl),
                            Err(err) => self.errors.push(err),
                        }
                    }
                    if terminator == Terminator::Brace {
                        self.pos += 1; // consume '}'
                        break;
                    }
                }
            }
        }
        declarations
    }

    #[doc = include_str!("stylesheet_parser.doc/collect_declaration.md")]
    fn collect_declaration(&mut self) -> (usize, Terminator) {
        let start = self.pos;
        let mut depth = 0i32;
        while let Some(kind) = self.cur_kind() {
            match kind {
                CssTokenKind::LParen | CssTokenKind::LBracket => depth += 1,
                CssTokenKind::RParen | CssTokenKind::RBracket => depth = depth.saturating_sub(1),
                CssTokenKind::Semicolon if depth == 0 => {
                    self.pos += 1; // consume ';'
                    return (start, Terminator::Semicolon);
                }
                CssTokenKind::RBrace if depth == 0 => return (start, Terminator::Brace),
                _ => {}
            }
            self.pos += 1;
        }
        (start, Terminator::Eof)
    }

    #[doc = include_str!("stylesheet_parser.doc/find_lbrace.md")]
    fn find_lbrace(&self) -> Option<usize> {
        let mut i = self.pos;
        let mut depth = 0i32;
        while let Some(tok) = self.tokens.get(i) {
            match tok.kind() {
                CssTokenKind::LParen | CssTokenKind::LBracket => depth += 1,
                CssTokenKind::RParen | CssTokenKind::RBracket => depth = depth.saturating_sub(1),
                CssTokenKind::LBrace if depth == 0 => return Some(i),
                CssTokenKind::RBrace | CssTokenKind::Semicolon if depth == 0 => return None,
                _ => {}
            }
            i += 1;
        }
        None
    }

    #[doc = include_str!("stylesheet_parser.doc/skip_block_body.md")]
    fn skip_block_body(&mut self) {
        let mut depth = 1i32;
        while let Some(kind) = self.cur_kind() {
            match kind {
                CssTokenKind::LBrace => depth += 1,
                CssTokenKind::RBrace => {
                    depth -= 1;
                    self.pos += 1;
                    if depth == 0 {
                        return;
                    }
                    continue;
                }
                _ => {}
            }
            self.pos += 1;
        }
    }

    #[doc = include_str!("stylesheet_parser.doc/skip_at_rule.md")]
    fn skip_at_rule(&mut self) {
        let mut depth = 0i32;
        while let Some(kind) = self.cur_kind() {
            match kind {
                CssTokenKind::LBrace => {
                    self.pos += 1;
                    self.skip_block_body();
                    return;
                }
                CssTokenKind::Semicolon if depth == 0 => {
                    self.pos += 1;
                    return;
                }
                CssTokenKind::LParen | CssTokenKind::LBracket => depth += 1,
                CssTokenKind::RParen | CssTokenKind::RBracket => depth = depth.saturating_sub(1),
                CssTokenKind::RBrace if depth == 0 => return,
                _ => {}
            }
            self.pos += 1;
        }
    }

    #[doc = include_str!("stylesheet_parser.doc/recover.md")]
    fn recover(&mut self) {
        let mut depth = 0i32;
        while let Some(kind) = self.cur_kind() {
            match kind {
                CssTokenKind::LBrace => {
                    self.pos += 1;
                    self.skip_block_body();
                    return;
                }
                CssTokenKind::Semicolon if depth == 0 => {
                    self.pos += 1;
                    return;
                }
                CssTokenKind::LParen | CssTokenKind::LBracket => depth += 1,
                CssTokenKind::RParen | CssTokenKind::RBracket => depth = depth.saturating_sub(1),
                CssTokenKind::RBrace if depth == 0 => return,
                _ => {}
            }
            self.pos += 1;
        }
    }
}

#[doc = include_str!("stylesheet_parser.doc/Terminator.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Terminator {
    Semicolon,
    Brace,
    Eof,
}

#[doc = include_str!("stylesheet_parser.doc/trim_semicolon.md")]
fn trim_semicolon(tokens: &[CssToken]) -> &[CssToken] {
    match tokens.last() {
        Some(t) if matches!(t.kind(), CssTokenKind::Semicolon) => &tokens[..tokens.len() - 1],
        _ => tokens,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_style_rule() {
        let src = CssSource::anonymous("a { color: red; }");
        let sheet = StylesheetParser::new(&src).parse();
        assert!(sheet.errors.is_empty());
        assert_eq!(sheet.rules.len(), 1);
        let ParsedRule::Style(rule) = &sheet.rules[0] else {
            panic!("expected style rule");
        };
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(&*rule.declarations[0].name, "color");
    }

    #[test]
    fn parses_multiple_declarations() {
        let src = CssSource::anonymous("a { color: red; width: 10px }");
        let sheet = StylesheetParser::new(&src).parse();
        let ParsedRule::Style(rule) = &sheet.rules[0] else {
            panic!("expected style rule");
        };
        assert_eq!(rule.declarations.len(), 2);
    }

    #[test]
    fn error_recovery_continues_after_bad_rule() {
        let src = CssSource::anonymous("a { color: red } b @@@ { } c { width: 1px }");
        let sheet = StylesheetParser::new(&src).parse();
        let style_rules = sheet
            .rules
            .iter()
            .filter(|r| matches!(r, ParsedRule::Style(_)))
            .count();
        assert!(style_rules >= 2, "expected at least 2 recovered rules");
        assert!(!sheet.errors.is_empty());
    }

    #[test]
    fn unsupported_at_rule_recorded() {
        let src = CssSource::anonymous("@import \"x.css\"; a { color: red }");
        let sheet = StylesheetParser::new(&src).parse();
        assert_eq!(sheet.unsupported.len(), 1);
        assert_eq!(sheet.unsupported[0].name(), "@import");
        assert!(
            sheet
                .rules
                .iter()
                .any(|r| matches!(r, ParsedRule::Style(_)))
        );
    }

    #[test]
    fn container_at_rule_is_reported_unsupported() {
        let src = CssSource::anonymous("@container (min-width: 400px) { a { color: red } }");
        let sheet = StylesheetParser::new(&src).parse();
        assert_eq!(sheet.unsupported.len(), 1);
        assert_eq!(sheet.unsupported[0].name(), "@container");
        assert_eq!(sheet.unsupported[0].kind(), UnsupportedFeatureKind::AtRule);
        // Reported, but still parsed: the rule is not dropped from the AST.
        assert!(
            sheet
                .rules
                .iter()
                .any(|r| matches!(r, ParsedRule::Container(_)))
        );
    }

    #[test]
    fn media_at_rule_is_not_reported_unsupported() {
        // `@media` really is wired; only `@container` is inert.
        let src = CssSource::anonymous("@media (min-width: 400px) { a { color: red } }");
        let sheet = StylesheetParser::new(&src).parse();
        assert!(sheet.unsupported.is_empty());
    }

    #[test]
    fn parses_nested_media_rule() {
        let src = CssSource::anonymous("@media screen { a { color: red } }");
        let sheet = StylesheetParser::new(&src).parse();
        let ParsedRule::Media(rule) = &sheet.rules[0] else {
            panic!("expected media rule");
        };
        assert_eq!(rule.rules.len(), 1);
    }

    #[test]
    fn parses_keyframes() {
        let src = CssSource::anonymous("@keyframes spin { from { opacity: 0 } to { opacity: 1 } }");
        let sheet = StylesheetParser::new(&src).parse();
        let ParsedRule::Keyframes(rule) = &sheet.rules[0] else {
            panic!("expected keyframes rule");
        };
        assert_eq!(&*rule.name, "spin");
        assert_eq!(rule.keyframes.len(), 2);
        assert_eq!(rule.keyframes[0].selectors, vec![KeyframeSelector::From]);
    }

    #[test]
    fn keyframes_percentage_selectors() {
        let src = CssSource::anonymous("@keyframes k { 0%, 50% { opacity: 1 } }");
        let sheet = StylesheetParser::new(&src).parse();
        let ParsedRule::Keyframes(rule) = &sheet.rules[0] else {
            panic!("expected keyframes rule");
        };
        assert_eq!(
            rule.keyframes[0].selectors,
            vec![
                KeyframeSelector::Percent(0.0),
                KeyframeSelector::Percent(50.0)
            ]
        );
    }

    #[test]
    fn empty_stylesheet() {
        let src = CssSource::anonymous("   \n  ");
        let sheet = StylesheetParser::new(&src).parse();
        assert!(sheet.rules.is_empty());
        assert!(sheet.errors.is_empty());
    }
}
