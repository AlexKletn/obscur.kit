use crate::lexer::{Token, TokenKind};
use crate::parser::{ParseError, ParseErrorKind, ParseEvent};
use crate::syntax::{SyntaxKind, SyntaxToken};

/// The mutable state of a single parse: the token slice, a cursor and the
/// growing event stream.
pub(super) struct ParseWalker<'t> {
    tokens: &'t [Token],
    pos: usize,
    events: Vec<ParseEvent>,
}

impl<'t> ParseWalker<'t> {
    pub(super) fn parse(tokens: &'t [Token]) -> Vec<ParseEvent> {
        let mut walker = Self::new(tokens);
        walker.parse_document();
        walker.events
    }

    fn new(tokens: &'t [Token]) -> Self {
        Self {
            tokens,
            pos: 0,
            events: Vec::new(),
        }
    }

    /// Kind of the token `n` positions ahead, or [`Eof`](TokenKind::Eof).
    fn nth_kind(&self, n: usize) -> TokenKind {
        self.tokens
            .get(self.pos + n)
            .map_or(TokenKind::Eof, Token::kind)
    }

    /// Kind of the current token.
    fn peek(&self) -> TokenKind {
        self.nth_kind(0)
    }

    /// Text of the token `n` positions ahead, or `""`.
    fn nth_text(&self, n: usize) -> &'t str {
        self.tokens.get(self.pos + n).map_or("", Token::text)
    }

    /// Byte offset of the current token (the `Eof` marker sits at `source.len()`).
    fn offset(&self) -> u32 {
        self.tokens.get(self.pos).map_or(0, Token::offset)
    }

    /// `true` once the cursor reached the `Eof` marker.
    fn at_end(&self) -> bool {
        self.peek().is_eof()
    }

    fn start(&mut self, kind: SyntaxKind) {
        self.events.push(ParseEvent::StartNode(kind));
    }

    fn finish(&mut self) {
        self.events.push(ParseEvent::FinishNode);
    }

    fn error(&mut self, kind: ParseErrorKind, message: impl Into<Box<str>>, offset: u32) {
        self.events
            .push(ParseEvent::Error(ParseError::new(kind, message, offset)));
    }

    /// Emits the current token as a leaf and advances. No-op at `Eof`.
    fn bump(&mut self) {
        if let Some(tok) = self.tokens.get(self.pos)
            && !tok.kind().is_eof()
        {
            self.events
                .push(ParseEvent::Token(SyntaxToken::from_lexer(tok)));
            self.pos += 1;
        }
    }

    /// Emits any run of whitespace trivia into the currently open node.
    fn bump_trivia(&mut self) {
        while self.peek() == TokenKind::Whitespace {
            self.bump();
        }
    }

    /// `document := (element | expression | text)*`
    fn parse_document(&mut self) {
        self.start(SyntaxKind::Document);
        while !self.at_end() {
            match self.peek() {
                TokenKind::LAngle => self.parse_element_or_stray(),
                TokenKind::LBrace => self.parse_expression(),
                _ => self.parse_text_node(),
            }
        }
        self.finish();
    }

    /// Dispatches a leading `<`: a real element (`<Ident …>`) or a stray `<`
    /// that is wrapped in an [`ErrorNode`](SyntaxKind::ErrorNode).
    fn parse_element_or_stray(&mut self) {
        if self.nth_kind(1) == TokenKind::Ident {
            self.parse_element();
        } else {
            let off = self.offset();
            self.start(SyntaxKind::ErrorNode);
            self.error(
                ParseErrorKind::ExpectedTagName,
                "expected a tag name after `<`",
                off,
            );
            self.bump(); // consume the stray `<` so the cursor advances
            self.finish();
        }
    }

    #[doc = include_str!("../../../docs/api/parser/parser.doc/parse_element.md")]
    fn parse_element(&mut self) {
        let name = self.nth_text(1).to_owned();
        let kind = classify_element_kind(&name);
        self.start(kind);

        let self_closing = self.parse_open_tag();
        if !self_closing {
            self.parse_children();
            self.parse_close_tag(&name);
        }

        self.finish();
    }

    /// `'<' Ident attribute* ('/>' | '>')` → an [`OpenTag`](SyntaxKind::OpenTag)
    /// node. Returns `true` when the tag is self-closing (`/>`).
    fn parse_open_tag(&mut self) -> bool {
        self.start(SyntaxKind::OpenTag);
        self.bump(); // `<`
        self.bump(); // tag name Ident

        let self_closing = loop {
            self.bump_trivia();
            match self.peek() {
                TokenKind::Ident => self.parse_attribute(),
                TokenKind::RAngle => {
                    self.bump();
                    break false;
                }
                TokenKind::Slash => {
                    self.bump(); // `/`
                    if self.peek() == TokenKind::RAngle {
                        self.bump(); // `>`
                    } else {
                        self.error(
                            ParseErrorKind::ExpectedClose,
                            "expected `>` after `/` in self-closing tag",
                            self.offset(),
                        );
                    }
                    break true;
                }
                // A `<` here means the `>` was forgotten; stop the tag and let
                // the element treat what follows as children.
                TokenKind::LAngle | TokenKind::Eof => {
                    self.error(
                        ParseErrorKind::ExpectedClose,
                        "expected `>` or `/>` to close the open tag",
                        self.offset(),
                    );
                    break false;
                }
                // Any other stray token inside a tag: wrap and recover.
                _ => {
                    let off = self.offset();
                    self.start(SyntaxKind::ErrorNode);
                    self.error(
                        ParseErrorKind::UnexpectedToken,
                        "unexpected token in open tag",
                        off,
                    );
                    self.bump();
                    self.finish();
                }
            }
        };

        self.finish(); // OpenTag
        self_closing
    }

    #[doc = include_str!("../../../docs/api/parser/parser.doc/parse_attribute.md")]
    fn parse_attribute(&mut self) {
        self.start(SyntaxKind::Attribute);
        self.bump(); // name Ident

        if self.peek() == TokenKind::Colon {
            self.bump(); // `:`
            if self.peek() == TokenKind::Ident {
                self.bump(); // namespaced name part
            }
        }

        self.bump_trivia();
        if self.peek() == TokenKind::Equals {
            self.bump(); // `=`
            self.bump_trivia();
            self.parse_attribute_value();
        }
        // A bare attribute (no `=`) is tolerated silently.

        self.finish();
    }

    /// `value := String | Number | Ident | '{' expr '}'` →
    /// an [`AttributeValue`](SyntaxKind::AttributeValue) node.
    fn parse_attribute_value(&mut self) {
        match self.peek() {
            TokenKind::LBrace => {
                self.start(SyntaxKind::AttributeValue);
                self.parse_expression();
                self.finish();
            }
            TokenKind::String | TokenKind::Number | TokenKind::Ident => {
                self.start(SyntaxKind::AttributeValue);
                self.bump();
                self.finish();
            }
            _ => {
                // No value present; report but do not consume — the open-tag
                // loop will handle whatever token this is (and always advances).
                self.error(
                    ParseErrorKind::ExpectedAttributeValue,
                    "expected a value after `=`",
                    self.offset(),
                );
            }
        }
    }

    #[doc = include_str!("../../../docs/api/parser/parser.doc/parse_expression.md")]
    fn parse_expression(&mut self) {
        let brace_off = self.offset();
        self.start(SyntaxKind::Expression);
        self.bump(); // `{`
        let mut depth: u32 = 1;
        while !self.at_end() {
            match self.peek() {
                TokenKind::LBrace => {
                    depth += 1;
                    self.bump();
                }
                TokenKind::RBrace => {
                    depth -= 1;
                    self.bump();
                    if depth == 0 {
                        break;
                    }
                }
                _ => self.bump(),
            }
        }
        if depth > 0 {
            self.error(
                ParseErrorKind::UnterminatedExpression,
                "expected `}` to close the expression",
                brace_off,
            );
        }
        self.finish();
    }

    /// A run of children between an element's open and close tags. Stops at the
    /// element's close tag (`</`) or end of input.
    fn parse_children(&mut self) {
        while !self.at_end() {
            match self.peek() {
                TokenKind::LAngle => {
                    if self.nth_kind(1) == TokenKind::Slash {
                        break; // close tag — handled by the caller
                    }
                    self.parse_element_or_stray();
                }
                TokenKind::LBrace => self.parse_expression(),
                _ => self.parse_text_node(),
            }
        }
    }

    /// `close := '<' '/' Ident '>'` → a [`CloseTag`](SyntaxKind::CloseTag) node.
    /// Reports a diagnostic (but still recovers) when the close tag is absent or
    /// its name does not match `expected`.
    fn parse_close_tag(&mut self, expected: &str) {
        if self.at_end() {
            self.error(
                ParseErrorKind::ExpectedClose,
                "expected a close tag before end of input",
                self.offset(),
            );
            return;
        }

        // Precondition from `parse_children`: `<` followed by `/`.
        self.start(SyntaxKind::CloseTag);
        self.bump(); // `<`
        self.bump(); // `/`
        self.bump_trivia();

        if self.peek() == TokenKind::Ident {
            let name_off = self.offset();
            let name = self.nth_text(0).to_owned();
            self.bump();
            if name != expected {
                self.error(
                    ParseErrorKind::MismatchedCloseTag,
                    format!("close tag `</{name}>` does not match `<{expected}>`"),
                    name_off,
                );
            }
        } else {
            self.error(
                ParseErrorKind::ExpectedTagName,
                "expected a tag name in close tag",
                self.offset(),
            );
        }

        self.bump_trivia();
        if self.peek() == TokenKind::RAngle {
            self.bump();
        } else {
            self.error(
                ParseErrorKind::ExpectedClose,
                "expected `>` to close the close tag",
                self.offset(),
            );
        }

        self.finish();
    }

    #[doc = include_str!("../../../docs/api/parser/parser.doc/parse_text_node.md")]
    fn parse_text_node(&mut self) {
        self.start(SyntaxKind::TextNode);
        while !self.at_end() && self.peek() != TokenKind::LAngle && self.peek() != TokenKind::LBrace
        {
            self.bump();
        }
        self.finish();
    }
}

/// Classifies an element by its tag name (see [`Parser`](super::Parser) docs). `If`/`ElseIf`/
/// `Else` → [`Conditional`](SyntaxKind::Conditional); `For`/`VirtualFor` →
/// [`ForLoop`](SyntaxKind::ForLoop); everything else → [`Element`](SyntaxKind::Element).
fn classify_element_kind(name: &str) -> SyntaxKind {
    match name {
        "If" | "ElseIf" | "Else" => SyntaxKind::Conditional,
        "For" | "VirtualFor" => SyntaxKind::ForLoop,
        _ => SyntaxKind::Element,
    }
}
