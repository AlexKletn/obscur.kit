use super::declaration_terminator::DeclarationTerminator;
use crate::error::{
    CssParseError, CssParseErrorKind, UnsupportedCssFeature, UnsupportedFeatureKind,
};
use crate::source::SourceSpan;

use super::super::{
    ContainerQueryParser, KeyframeSelector, MediaQueryParser, ParsedContainerQuery,
    ParsedContainerRule, ParsedDeclaration, ParsedKeyframe, ParsedKeyframesRule, ParsedMediaRule,
    ParsedRule, ParsedStyleRule, ParsedStylesheet, PropertyParser, SelectorParser,
};
use super::super::{css_token::CssToken, css_token_kind::CssTokenKind, css_tokenizer::Tokenizer};

/// The mutable state driving a single stylesheet parse.
pub(super) struct StylesheetBuilder<'a> {
    text: &'a str,
    tokens: Vec<CssToken>,
    pos: usize,
    eof: u32,
    errors: Vec<CssParseError>,
    unsupported: Vec<UnsupportedCssFeature>,
}

impl<'a> StylesheetBuilder<'a> {
    pub(super) fn parse(text: &'a str) -> ParsedStylesheet {
        let eof = u32::try_from(text.len()).unwrap_or(u32::MAX);
        let (tokens, tok_errors) = Tokenizer::new(text).tokenize();
        let mut builder = Self {
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
            // Every turn of this loop must consume something. A recovery path
            // that stops without stepping over the token it choked on turns a
            // one-character typo into an unbounded loop that allocates a fresh
            // error each time round — which is a stylesheet taking the process
            // down, not a diagnostic. The arms below all advance; this is the
            // net under them, so that no future one has to be trusted to.
            let entry = self.pos;
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
                // A stray `}` at the top level closes nothing. It is CONSUMED
                // here, because the recovery path below stops in front of a
                // closing brace without stepping over it — leaving this to that
                // path spun forever, pushing one error per turn until the
                // process ran out of memory. One `}` is one fault.
                Some(CssTokenKind::RBrace) => {
                    self.errors.push(CssParseError::new(
                        CssParseErrorKind::UnexpectedToken,
                        "stray '}' closes no block",
                        self.cur_span(),
                    ));
                    self.pos += 1;
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
            if self.pos == entry {
                self.pos += 1;
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
        let (query, errors) =
            MediaQueryParser::parse_tokens_recovering(prelude, self.tokens[lbrace].span().start());
        self.errors.extend(errors);
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
        Some(ParsedRule::Keyframes(ParsedKeyframesRule {
            name,
            keyframes,
        }))
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
                    if terminator == DeclarationTerminator::Brace {
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
    fn collect_declaration(&mut self) -> (usize, DeclarationTerminator) {
        let start = self.pos;
        let mut depth = 0i32;
        while let Some(kind) = self.cur_kind() {
            match kind {
                CssTokenKind::LParen | CssTokenKind::LBracket => depth += 1,
                CssTokenKind::RParen | CssTokenKind::RBracket => depth = depth.saturating_sub(1),
                CssTokenKind::Semicolon if depth == 0 => {
                    self.pos += 1; // consume ';'
                    return (start, DeclarationTerminator::Semicolon);
                }
                CssTokenKind::RBrace if depth == 0 => return (start, DeclarationTerminator::Brace),
                _ => {}
            }
            self.pos += 1;
        }
        (start, DeclarationTerminator::Eof)
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

/// Drops a single trailing `;` token from a declaration slice, if present.
fn trim_semicolon(tokens: &[CssToken]) -> &[CssToken] {
    match tokens.last() {
        Some(t) if matches!(t.kind(), CssTokenKind::Semicolon) => &tokens[..tokens.len() - 1],
        _ => tokens,
    }
}
