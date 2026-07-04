//! The CSS tokenizer and the top-level [`StylesheetParser`] entry point.
//!
//! This file owns the crate's hand-rolled CSS tokenizer (a simplified CSS
//! Syntax Level 3 token model, spec §4.13) and the resilient top-level parser
//! that turns a [`CssSource`] into a [`ParsedStylesheet`]. The tokenizer and the
//! shared [`TokenCursor`] are `pub(crate)`: the sibling parsers
//! ([`super::selector_parser`], [`super::value_parser`], and friends) consume
//! the same token stream so a stylesheet is tokenized exactly once.

use crate::error::{
    CssParseError, CssParseErrorKind, UnsupportedCssFeature, UnsupportedFeatureKind,
};
use crate::source::{CssSource, SourceSpan};

use super::container_query_parser::{ContainerQueryParser, ParsedContainerQuery};
use super::media_query_parser::{MediaQueryParser, ParsedMediaQueryList};
use super::property_parser::{ParsedDeclaration, PropertyParser};
use super::selector_parser::{ParsedSelectorList, SelectorParser};

// --------------------------------------------------------------------------
// Token model
// --------------------------------------------------------------------------

/// The lexical category of a [`CssToken`] (CSS Syntax Level 3, simplified).
///
/// Each variant that carries text stores it already stripped of its delimiters
/// (a [`Hash`](CssTokenKind::Hash) drops the leading `#`, a
/// [`Str`](CssTokenKind::Str) drops the surrounding quotes, an
/// [`AtKeyword`](CssTokenKind::AtKeyword) drops the leading `@`).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CssTokenKind {
    /// An identifier such as `flex`, `--custom`, or `-webkit-box`.
    Ident(Box<str>),
    /// An at-keyword such as `@media`; stores `media` (without the `@`).
    AtKeyword(Box<str>),
    /// A hash token such as `#id` or `#ff0000`; stores the name (without `#`).
    Hash(Box<str>),
    /// A quoted string; stores the inner text (without the quotes).
    Str(Box<str>),
    /// A unitless number such as `12` or `-3.5`.
    Number(f64),
    /// A percentage such as `50%`; stores the numeric part (`50`).
    Percentage(f64),
    /// A dimension such as `12px`; stores the number and the unit text.
    Dimension {
        /// The numeric part of the dimension.
        value: f64,
        /// The unit text (for example `px`, `rem`, `deg`).
        unit: Box<str>,
    },
    /// A single delimiter or operator character (`> + ~ * = | ^ $ . ! / &` …).
    Delim(char),
    /// A `:` colon.
    Colon,
    /// A `;` semicolon.
    Semicolon,
    /// A `,` comma.
    Comma,
    /// A `{` left brace.
    LBrace,
    /// A `}` right brace.
    RBrace,
    /// A `(` left parenthesis.
    LParen,
    /// A `)` right parenthesis.
    RParen,
    /// A `[` left bracket.
    LBracket,
    /// A `]` right bracket.
    RBracket,
    /// A maximal run of collapsible whitespace.
    Whitespace,
}

/// A single CSS token: a [`CssTokenKind`] together with the [`SourceSpan`] it
/// occupies in the originating source.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CssToken {
    kind: CssTokenKind,
    span: SourceSpan,
}

impl CssToken {
    /// Creates a token of `kind` covering `span`.
    pub(crate) fn new(kind: CssTokenKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }

    /// Returns the token's lexical kind.
    pub(crate) fn kind(&self) -> &CssTokenKind {
        &self.kind
    }

    /// Returns the span the token occupies.
    pub(crate) fn span(&self) -> SourceSpan {
        self.span
    }

    /// Returns whether this token is whitespace.
    pub(crate) fn is_whitespace(&self) -> bool {
        matches!(self.kind, CssTokenKind::Whitespace)
    }
}

/// Returns whether `c` may start a CSS identifier.
fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || (c as u32) >= 0x80
}

/// Returns whether `c` may continue a CSS identifier.
fn is_ident_continue(c: char) -> bool {
    is_ident_start(c) || c.is_ascii_digit() || c == '-'
}

// --------------------------------------------------------------------------
// Tokenizer
// --------------------------------------------------------------------------

/// A hand-rolled CSS tokenizer (spec §4.13).
///
/// The tokenizer walks the source by [`char_indices`](str::char_indices), so it
/// never splits a UTF-8 character, and produces byte-accurate [`SourceSpan`]s.
/// It is *total*: it never panics. Comments (`/* … */`) are skipped; an
/// unterminated string or comment produces a recorded [`CssParseError`] while
/// still yielding a best-effort token stream so higher layers can recover.
pub(crate) struct Tokenizer<'a> {
    text: &'a str,
    chars: Vec<(usize, char)>,
    pos: usize,
    errors: Vec<CssParseError>,
}

impl<'a> Tokenizer<'a> {
    /// Creates a tokenizer over `text`.
    pub(crate) fn new(text: &'a str) -> Self {
        Self {
            text,
            chars: text.char_indices().collect(),
            pos: 0,
            errors: Vec::new(),
        }
    }

    /// Consumes the tokenizer, returning the token stream and any errors
    /// collected (unterminated strings or comments).
    pub(crate) fn tokenize(mut self) -> (Vec<CssToken>, Vec<CssParseError>) {
        let mut tokens = Vec::new();
        while self.pos < self.chars.len() {
            if let Some(tok) = self.next_token() {
                tokens.push(tok);
            }
        }
        (tokens, self.errors)
    }

    /// Returns the char at the current position.
    fn cur(&self) -> Option<char> {
        self.chars.get(self.pos).map(|&(_, c)| c)
    }

    /// Returns the char `k` positions ahead of the current position.
    fn peek(&self, k: usize) -> Option<char> {
        self.chars.get(self.pos + k).map(|&(_, c)| c)
    }

    /// Returns the byte offset for char index `idx` (or the text length at end).
    fn byte(&self, idx: usize) -> u32 {
        let b = self.chars.get(idx).map_or(self.text.len(), |&(b, _)| b);
        u32::try_from(b).unwrap_or(u32::MAX)
    }

    /// Returns the substring covered by char indices `[start, end)`.
    fn slice(&self, start: usize, end: usize) -> &str {
        let a = self.chars.get(start).map_or(self.text.len(), |&(b, _)| b);
        let b = self.chars.get(end).map_or(self.text.len(), |&(b, _)| b);
        &self.text[a..b]
    }

    /// Builds a span covering char indices `[start, end)`.
    fn span(&self, start: usize, end: usize) -> SourceSpan {
        SourceSpan::new(self.byte(start), self.byte(end))
    }

    /// Scans the next token, or [`None`] for skipped input (comments).
    fn next_token(&mut self) -> Option<CssToken> {
        let start = self.pos;
        let c = self.cur()?;

        if c.is_whitespace() {
            while matches!(self.cur(), Some(ch) if ch.is_whitespace()) {
                self.pos += 1;
            }
            return Some(self.emit(CssTokenKind::Whitespace, start));
        }
        if c == '/' && self.peek(1) == Some('*') {
            self.skip_comment(start);
            return None;
        }
        if c == '"' || c == '\'' {
            return Some(self.scan_string(c, start));
        }
        if self.is_number_start() {
            let kind = self.scan_number();
            return Some(self.emit(kind, start));
        }
        if self.is_ident_start_here() {
            let name = self.scan_ident();
            return Some(self.emit(CssTokenKind::Ident(name), start));
        }
        if c == '@' {
            self.pos += 1;
            if self.is_ident_start_here() {
                let name = self.scan_ident();
                return Some(self.emit(CssTokenKind::AtKeyword(name), start));
            }
            return Some(self.emit(CssTokenKind::Delim('@'), start));
        }
        if c == '#' {
            self.pos += 1;
            if matches!(self.cur(), Some(ch) if is_ident_continue(ch)) {
                let name = self.scan_name();
                return Some(self.emit(CssTokenKind::Hash(name), start));
            }
            return Some(self.emit(CssTokenKind::Delim('#'), start));
        }

        self.pos += 1;
        let kind = match c {
            ':' => CssTokenKind::Colon,
            ';' => CssTokenKind::Semicolon,
            ',' => CssTokenKind::Comma,
            '{' => CssTokenKind::LBrace,
            '}' => CssTokenKind::RBrace,
            '(' => CssTokenKind::LParen,
            ')' => CssTokenKind::RParen,
            '[' => CssTokenKind::LBracket,
            ']' => CssTokenKind::RBracket,
            other => CssTokenKind::Delim(other),
        };
        Some(self.emit(kind, start))
    }

    /// Emits a token of `kind` spanning from char index `start` to the current
    /// position.
    fn emit(&self, kind: CssTokenKind, start: usize) -> CssToken {
        CssToken::new(kind, self.span(start, self.pos))
    }

    /// Skips a `/* … */` comment; records an error if it is unterminated.
    fn skip_comment(&mut self, start: usize) {
        self.pos += 2;
        while self.pos < self.chars.len() {
            if self.cur() == Some('*') && self.peek(1) == Some('/') {
                self.pos += 2;
                return;
            }
            self.pos += 1;
        }
        self.errors.push(CssParseError::new(
            CssParseErrorKind::UnexpectedEof,
            "unterminated comment",
            self.span(start, self.pos),
        ));
    }

    /// Scans a quoted string opened by `quote`; records an error and runs to
    /// end-of-input if unterminated.
    fn scan_string(&mut self, quote: char, start: usize) -> CssToken {
        self.pos += 1;
        let content_start = self.pos;
        while let Some(c) = self.cur() {
            if c == '\\' {
                self.pos += 2;
                continue;
            }
            if c == quote {
                let content: Box<str> = self.slice(content_start, self.pos).into();
                self.pos += 1;
                return self.emit(CssTokenKind::Str(content), start);
            }
            self.pos += 1;
        }
        // Unterminated: emit the run to end-of-input and record an error.
        let content: Box<str> = self.slice(content_start, self.pos).into();
        self.errors.push(CssParseError::new(
            CssParseErrorKind::UnterminatedString,
            "unterminated string literal",
            self.span(start, self.pos),
        ));
        self.emit(CssTokenKind::Str(content), start)
    }

    /// Returns whether an identifier starts at the current position (including
    /// leading hyphens for custom properties and vendor prefixes).
    fn is_ident_start_here(&self) -> bool {
        match self.cur() {
            Some(c) if is_ident_start(c) => true,
            Some('-') => matches!(self.peek(1), Some(d) if is_ident_start(d) || d == '-'),
            _ => false,
        }
    }

    /// Returns whether a number starts at the current position.
    fn is_number_start(&self) -> bool {
        match self.cur() {
            Some(c) if c.is_ascii_digit() => true,
            Some('.') => matches!(self.peek(1), Some(d) if d.is_ascii_digit()),
            Some('+' | '-') => match self.peek(1) {
                Some(d) if d.is_ascii_digit() => true,
                Some('.') => matches!(self.peek(2), Some(d) if d.is_ascii_digit()),
                _ => false,
            },
            _ => false,
        }
    }

    /// Scans an identifier (already known to start here) into owned text.
    fn scan_ident(&mut self) -> Box<str> {
        let start = self.pos;
        self.pos += 1;
        while matches!(self.cur(), Some(c) if is_ident_continue(c)) {
            self.pos += 1;
        }
        self.slice(start, self.pos).into()
    }

    /// Scans a hash/name run (letters, digits, `-`, `_`) into owned text.
    fn scan_name(&mut self) -> Box<str> {
        let start = self.pos;
        while matches!(self.cur(), Some(c) if is_ident_continue(c)) {
            self.pos += 1;
        }
        self.slice(start, self.pos).into()
    }

    /// Scans a number, returning [`Number`](CssTokenKind::Number),
    /// [`Percentage`](CssTokenKind::Percentage), or
    /// [`Dimension`](CssTokenKind::Dimension) depending on the trailing unit.
    fn scan_number(&mut self) -> CssTokenKind {
        let start = self.pos;
        if matches!(self.cur(), Some('+' | '-')) {
            self.pos += 1;
        }
        while matches!(self.cur(), Some(c) if c.is_ascii_digit()) {
            self.pos += 1;
        }
        if self.cur() == Some('.') && matches!(self.peek(1), Some(c) if c.is_ascii_digit()) {
            self.pos += 1;
            while matches!(self.cur(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        if matches!(self.cur(), Some('e' | 'E')) {
            let sign = usize::from(matches!(self.peek(1), Some('+' | '-')));
            if matches!(self.peek(1 + sign), Some(c) if c.is_ascii_digit()) {
                self.pos += 1 + sign;
                while matches!(self.cur(), Some(c) if c.is_ascii_digit()) {
                    self.pos += 1;
                }
            }
        }
        let value = self.slice(start, self.pos).parse::<f64>().unwrap_or(0.0);
        if self.cur() == Some('%') {
            self.pos += 1;
            return CssTokenKind::Percentage(value);
        }
        if matches!(self.cur(), Some(c) if is_ident_start(c)) {
            let unit_start = self.pos;
            self.pos += 1;
            while matches!(self.cur(), Some(c) if is_ident_continue(c)) {
                self.pos += 1;
            }
            let unit: Box<str> = self.slice(unit_start, self.pos).into();
            return CssTokenKind::Dimension { value, unit };
        }
        CssTokenKind::Number(value)
    }
}

// --------------------------------------------------------------------------
// Shared token cursor
// --------------------------------------------------------------------------

/// A forward cursor over a token slice, shared by every sub-parser.
///
/// The cursor exposes whitespace-aware navigation: whitespace tokens remain in
/// the stream (selectors need them as descendant combinators) but can be skipped
/// on demand. It records the end-of-input byte offset so parsers can raise
/// [`CssParseError::unexpected_eof`] with an accurate location.
pub(crate) struct TokenCursor<'t> {
    tokens: &'t [CssToken],
    pos: usize,
    eof: u32,
}

impl<'t> TokenCursor<'t> {
    /// Creates a cursor over `tokens`, reporting `eof` as the end-of-input byte
    /// offset.
    pub(crate) fn new(tokens: &'t [CssToken], eof: u32) -> Self {
        Self {
            tokens,
            pos: 0,
            eof,
        }
    }

    /// Returns the current token without advancing.
    pub(crate) fn peek(&self) -> Option<&'t CssToken> {
        self.tokens.get(self.pos)
    }

    /// Returns the kind of the current token without advancing.
    pub(crate) fn peek_kind(&self) -> Option<&'t CssTokenKind> {
        self.tokens.get(self.pos).map(CssToken::kind)
    }

    /// Returns the current token and advances past it.
    pub(crate) fn advance(&mut self) -> Option<&'t CssToken> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    /// Advances past any run of whitespace tokens.
    pub(crate) fn skip_ws(&mut self) {
        while matches!(self.tokens.get(self.pos), Some(t) if t.is_whitespace()) {
            self.pos += 1;
        }
    }

    /// Returns the end-of-input byte offset.
    pub(crate) fn eof(&self) -> u32 {
        self.eof
    }

    /// Returns the span of the current token, or an empty span at end-of-input.
    pub(crate) fn here_span(&self) -> SourceSpan {
        self.tokens
            .get(self.pos)
            .map_or_else(|| SourceSpan::empty(self.eof), CssToken::span)
    }
}

/// Returns a span covering `tokens`, or an empty span at `eof` when empty.
pub(crate) fn slice_span(tokens: &[CssToken], eof: u32) -> SourceSpan {
    match (tokens.first(), tokens.last()) {
        (Some(first), Some(last)) => first.span().union(last.span()),
        _ => SourceSpan::empty(eof),
    }
}

// --------------------------------------------------------------------------
// Top-level parsed AST
// --------------------------------------------------------------------------

/// A fully parsed CSS stylesheet with recovered rules and diagnostics.
///
/// Parsing is resilient: a malformed rule is recorded in `errors` and skipped,
/// and recognised-but-unsupported at-rules (for example `@import`) are recorded
/// in `unsupported` rather than aborting the parse.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedStylesheet {
    /// The recovered top-level rules, in source order.
    pub rules: Vec<ParsedRule>,
    /// Syntax errors encountered (each rule after an error is still attempted).
    pub errors: Vec<CssParseError>,
    /// Recognised-but-unsupported at-rules encountered during parsing.
    pub unsupported: Vec<UnsupportedCssFeature>,
}

/// A single top-level (or nested) CSS rule.
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedRule {
    /// A style rule: a selector list and its declaration block.
    Style(ParsedStyleRule),
    /// An `@media` rule with a nested rule list.
    Media(ParsedMediaRule),
    /// An `@container` rule with a nested rule list.
    Container(ParsedContainerRule),
    /// An `@keyframes` rule.
    Keyframes(ParsedKeyframesRule),
}

/// A style rule: a selector list paired with its declarations.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedStyleRule {
    /// The rule's selector list.
    pub selectors: ParsedSelectorList,
    /// The declarations in the rule's block.
    pub declarations: Vec<ParsedDeclaration>,
}

/// An `@media` rule: a media-query list wrapping a nested rule list.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedMediaRule {
    /// The media-query list from the rule's prelude.
    pub query: ParsedMediaQueryList,
    /// The nested rules the media block applies to.
    pub rules: Vec<ParsedRule>,
}

/// An `@container` rule: a container query wrapping a nested rule list.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedContainerRule {
    /// The container query from the rule's prelude.
    pub query: ParsedContainerQuery,
    /// The nested rules the container block applies to.
    pub rules: Vec<ParsedRule>,
}

/// An `@keyframes` rule: a name and its ordered keyframes.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedKeyframesRule {
    /// The animation name.
    pub name: Box<str>,
    /// The keyframes, in source order.
    pub keyframes: Vec<ParsedKeyframe>,
}

/// One keyframe within a [`ParsedKeyframesRule`].
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedKeyframe {
    /// The keyframe selectors (`from`, `to`, or percentage offsets).
    pub selectors: Vec<KeyframeSelector>,
    /// The declarations applied at this keyframe.
    pub declarations: Vec<ParsedDeclaration>,
}

/// A single keyframe offset selector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyframeSelector {
    /// The `from` keyword (equivalent to `0%`).
    From,
    /// The `to` keyword (equivalent to `100%`).
    To,
    /// An explicit percentage offset (for example `50` for `50%`).
    Percent(f64),
}

// --------------------------------------------------------------------------
// Stylesheet parser
// --------------------------------------------------------------------------

/// The top-level CSS parser: turns a [`CssSource`] into a [`ParsedStylesheet`].
///
/// The parser is resilient (spec §4.13): a malformed rule is recorded and the
/// parser recovers to the next rule boundary, so a single mistake never discards
/// the rest of the stylesheet.
#[derive(Debug)]
pub struct StylesheetParser<'a> {
    source: &'a CssSource,
}

impl<'a> StylesheetParser<'a> {
    /// Creates a parser for `source`.
    #[must_use]
    pub fn new(source: &'a CssSource) -> Self {
        Self { source }
    }

    /// Parses the source into a [`ParsedStylesheet`], recovering from errors.
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

/// The mutable state driving a single stylesheet parse.
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

    /// Parses a list of rules. When `nested`, stops at (and consumes) the
    /// closing `}`; otherwise stops at end-of-input.
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

    /// Parses a style rule (selector list + declaration block), recovering on
    /// error. Returns [`None`] when the rule is dropped during recovery.
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

    /// Parses an at-rule prelude and body, dispatching on the keyword.
    fn parse_at_rule(&mut self) -> Option<ParsedRule> {
        let at_span = self.cur_span();
        let Some(CssTokenKind::AtKeyword(name)) = self.cur_kind() else {
            return None;
        };
        let name = name.to_ascii_lowercase();
        self.pos += 1; // consume the at-keyword

        match name.as_str() {
            "media" => self.parse_media_rule(),
            "container" => self.parse_container_rule(),
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

    /// Parses an `@media` rule.
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

    /// Parses an `@container` rule.
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

    /// Parses an `@keyframes` rule.
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
        Some(ParsedRule::Keyframes(ParsedKeyframesRule { name, keyframes }))
    }

    /// Parses a single keyframe (`0%, 50% { … }`).
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

    /// Interprets the keyframe selector tokens in `[start, lbrace)`.
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

    /// Parses declarations until the matching `}` (which is consumed).
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

    /// Advances past one declaration's tokens, stopping at (and consuming) a
    /// `;`, or stopping before a block-closing `}`. Returns the start index and
    /// which terminator ended the declaration.
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

    /// Finds the next top-level `{`, returning its index; returns [`None`] if a
    /// `}`, `;`, or end-of-input is reached first (a malformed rule).
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

    /// Skips a balanced `{ … }` body, assuming the opening `{` was already
    /// consumed.
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

    /// Skips an at-rule: a balanced `{ … }` block if one follows, else up to and
    /// including the next `;` at top level.
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

    /// Recovers from an error by skipping to the next rule boundary: a balanced
    /// `{ … }` block, a top-level `;`, or a top-level `}`.
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

/// Which token terminated a collected declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Terminator {
    Semicolon,
    Brace,
    Eof,
}

/// Drops a single trailing `;` token from a declaration slice, if present.
fn trim_semicolon(tokens: &[CssToken]) -> &[CssToken] {
    match tokens.last() {
        Some(t) if matches!(t.kind(), CssTokenKind::Semicolon) => &tokens[..tokens.len() - 1],
        _ => tokens,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(text: &str) -> Vec<CssToken> {
        Tokenizer::new(text).tokenize().0
    }

    fn kinds(text: &str) -> Vec<CssTokenKind> {
        toks(text).into_iter().map(|t| t.kind().clone()).collect()
    }

    #[test]
    fn tokenizes_identifiers_and_custom_props() {
        assert_eq!(kinds("flex"), vec![CssTokenKind::Ident("flex".into())]);
        assert_eq!(
            kinds("--main-color"),
            vec![CssTokenKind::Ident("--main-color".into())]
        );
        assert_eq!(
            kinds("-webkit-box"),
            vec![CssTokenKind::Ident("-webkit-box".into())]
        );
    }

    #[test]
    fn tokenizes_at_keyword_and_hash() {
        assert_eq!(kinds("@media"), vec![CssTokenKind::AtKeyword("media".into())]);
        assert_eq!(kinds("#id"), vec![CssTokenKind::Hash("id".into())]);
        assert_eq!(kinds("#ff0000"), vec![CssTokenKind::Hash("ff0000".into())]);
    }

    #[test]
    fn tokenizes_numbers_percentages_dimensions() {
        assert_eq!(kinds("12"), vec![CssTokenKind::Number(12.0)]);
        assert_eq!(kinds("-3.5"), vec![CssTokenKind::Number(-3.5)]);
        assert_eq!(kinds("50%"), vec![CssTokenKind::Percentage(50.0)]);
        assert_eq!(
            kinds("12px"),
            vec![CssTokenKind::Dimension {
                value: 12.0,
                unit: "px".into()
            }]
        );
        assert_eq!(
            kinds("1.5em"),
            vec![CssTokenKind::Dimension {
                value: 1.5,
                unit: "em".into()
            }]
        );
    }

    #[test]
    fn dot_before_digit_is_number() {
        assert_eq!(kinds(".5"), vec![CssTokenKind::Number(0.5)]);
        assert_eq!(
            kinds(".foo"),
            vec![CssTokenKind::Delim('.'), CssTokenKind::Ident("foo".into())]
        );
    }

    #[test]
    fn scientific_notation_vs_unit() {
        assert_eq!(kinds("1e3"), vec![CssTokenKind::Number(1000.0)]);
        assert_eq!(
            kinds("1em"),
            vec![CssTokenKind::Dimension {
                value: 1.0,
                unit: "em".into()
            }]
        );
    }

    #[test]
    fn tokenizes_strings_both_quotes() {
        assert_eq!(kinds("\"hi\""), vec![CssTokenKind::Str("hi".into())]);
        assert_eq!(kinds("'yo'"), vec![CssTokenKind::Str("yo".into())]);
    }

    #[test]
    fn unterminated_string_records_error() {
        let (tokens, errors) = Tokenizer::new("\"oops").tokenize();
        assert_eq!(tokens.len(), 1);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind(), CssParseErrorKind::UnterminatedString);
    }

    #[test]
    fn comments_are_skipped() {
        assert_eq!(
            kinds("a/* c */b"),
            vec![
                CssTokenKind::Ident("a".into()),
                CssTokenKind::Ident("b".into())
            ]
        );
    }

    #[test]
    fn unterminated_comment_records_error() {
        let (_, errors) = Tokenizer::new("a /* unclosed").tokenize();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind(), CssParseErrorKind::UnexpectedEof);
    }

    #[test]
    fn punctuation_and_delims() {
        assert_eq!(
            kinds("{}();:,>+~"),
            vec![
                CssTokenKind::LBrace,
                CssTokenKind::RBrace,
                CssTokenKind::LParen,
                CssTokenKind::RParen,
                CssTokenKind::Semicolon,
                CssTokenKind::Colon,
                CssTokenKind::Comma,
                CssTokenKind::Delim('>'),
                CssTokenKind::Delim('+'),
                CssTokenKind::Delim('~'),
            ]
        );
    }

    #[test]
    fn spans_are_byte_accurate() {
        let tokens = toks("ab cd");
        assert_eq!(tokens[0].span(), SourceSpan::new(0, 2));
        assert!(tokens[1].is_whitespace());
        assert_eq!(tokens[2].span(), SourceSpan::new(3, 5));
    }

    #[test]
    fn multibyte_is_char_boundary_safe() {
        let tokens = toks("\"é\" a");
        assert_eq!(tokens[0].kind(), &CssTokenKind::Str("é".into()));
        let last = tokens.last().unwrap();
        assert_eq!(last.kind(), &CssTokenKind::Ident("a".into()));
    }

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
        assert!(sheet.rules.iter().any(|r| matches!(r, ParsedRule::Style(_))));
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
        let src =
            CssSource::anonymous("@keyframes spin { from { opacity: 0 } to { opacity: 1 } }");
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
