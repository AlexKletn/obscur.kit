//! `Parser`: an error-tolerant recursive-descent parser for `.okui` source
//! (spec §2.9) that always yields a lossless [`SyntaxTree`].

use crate::lexer::{Lexer, Token, TokenKind};
use crate::parser::{ParseError, ParseErrorKind, ParseEvent, ParseResult};
use crate::syntax::{SyntaxKind, SyntaxNode, SyntaxToken, SyntaxTree};

/// A recursive-descent parser for the `.okui` markup language (spec §2.9).
///
/// The parser turns source text into a lossless [`SyntaxTree`] paired with any
/// recoverable diagnostics ([`ParseResult`]). It is *total*: it never panics,
/// never hangs, and never drops a byte — `result.tree().text()` always equals
/// the input, malformed or not.
///
/// # Algorithm
///
/// [`Parser::parse`] runs in three phases:
///
/// 1. **Lex.** [`Lexer::tokenize`] produces a lossless [`Token`] stream ending
///    in an [`Eof`](TokenKind::Eof) marker.
/// 2. **Walk.** A recursive-descent walk over the token slice (with a `usize`
///    cursor) emits a flat [`ParseEvent`] stream —
///    [`StartNode`](ParseEvent::StartNode) / [`Token`](ParseEvent::Token) /
///    [`FinishNode`](ParseEvent::FinishNode) / [`Error`](ParseEvent::Error).
///    *Every* token, including whitespace trivia and stray/erroneous tokens, is
///    emitted into the currently open node, which is what keeps the tree
///    lossless.
/// 3. **Materialize.** [`build_tree`] folds the balanced event stream into a
///    [`SyntaxNode`] tree using an explicit node stack, collecting the
///    [`Error`](ParseEvent::Error) events into the diagnostics vector.
///
/// # Grammar
///
/// ```text
/// document  := (element | expression | text)*                 -> Document
/// element   := '<' Ident attribute* ('/>' | '>' child* close) -> Element
/// close     := '<' '/' Ident '>'                              -> CloseTag
/// attribute := Ident (':' Ident)? ('=' value)?                -> Attribute
/// value     := String | Number | Ident | '{' expr '}'        -> AttributeValue
/// expression:= '{' expr_tokens '}'                            -> Expression
/// text      := (any run of non-`<`, non-`{` tokens)          -> TextNode
/// ```
///
/// The open tag `<Name …>` becomes an [`OpenTag`](SyntaxKind::OpenTag) child
/// node and the close `</Name>` a [`CloseTag`](SyntaxKind::CloseTag) child
/// node; both live directly inside the element node alongside its children.
///
/// ## Control-flow classification
///
/// The element node is labelled [`Conditional`](SyntaxKind::Conditional) when
/// its tag name is `If`, `ElseIf` or `Else`, and [`ForLoop`](SyntaxKind::ForLoop)
/// when it is `For` or `VirtualFor`; every other tag yields a plain
/// [`Element`](SyntaxKind::Element). This is only a hint — the compiler performs
/// the authoritative classification later.
///
/// ## Error recovery
///
/// The parser never aborts. On an unexpected token it records a [`ParseError`],
/// wraps the offending token(s) in an [`ErrorNode`](SyntaxKind::ErrorNode) and
/// advances so the cursor always makes progress. A missing or mismatched close
/// tag yields an [`ExpectedClose`](ParseErrorKind::ExpectedClose) /
/// [`MismatchedCloseTag`](ParseErrorKind::MismatchedCloseTag) diagnostic but the
/// element node is still finished. Because every token is emitted regardless,
/// losslessness holds even for malformed input.
#[derive(Debug, Clone, Default)]
pub struct Parser {
    _private: (),
}

impl Parser {
    /// Creates a new parser.
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// Parses `source` into a lossless [`ParseResult`].
    ///
    /// The returned tree always round-trips the input
    /// (`result.tree().text() == source`); any problems are reported in
    /// `result.errors()`.
    #[must_use]
    pub fn parse(&self, source: &str) -> ParseResult {
        let tokens = Lexer::new().tokenize(source);
        let mut walker = Walker::new(&tokens);
        walker.parse_document();
        let (root, errors) = build_tree(walker.events);
        ParseResult::new(SyntaxTree::new(root), errors)
    }
}

/// The mutable state of a single parse: the token slice, a cursor and the
/// growing event stream.
struct Walker<'t> {
    tokens: &'t [Token],
    pos: usize,
    events: Vec<ParseEvent>,
}

impl<'t> Walker<'t> {
    fn new(tokens: &'t [Token]) -> Self {
        Self {
            tokens,
            pos: 0,
            events: Vec::new(),
        }
    }

    // ---- cursor primitives -------------------------------------------------

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

    // ---- event helpers -----------------------------------------------------

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

    // ---- grammar -----------------------------------------------------------

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

    /// `element := '<' Ident attribute* ('/>' | '>' child* close)`
    ///
    /// Precondition: `peek() == LAngle` and the next token is an `Ident`.
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

    /// `attribute := Ident (':' Ident)? ('=' value)?`
    ///
    /// Precondition: `peek() == Ident`.
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

    /// `expression := '{' expr_tokens '}'` → an [`Expression`](SyntaxKind::Expression)
    /// node. Captures every token up to the matching `}`, tracking brace depth.
    ///
    /// Precondition: `peek() == LBrace`.
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

    /// `text := (run of non-`<`, non-`{` tokens)` → a [`TextNode`](SyntaxKind::TextNode).
    ///
    /// Absorbs raw text, whitespace and any stray non-markup tokens so nothing
    /// is lost. Always consumes at least one token when called on a non-markup
    /// token, guaranteeing forward progress.
    fn parse_text_node(&mut self) {
        self.start(SyntaxKind::TextNode);
        while !self.at_end()
            && self.peek() != TokenKind::LAngle
            && self.peek() != TokenKind::LBrace
        {
            self.bump();
        }
        self.finish();
    }
}

/// Classifies an element by its tag name (see [`Parser`] docs). `If`/`ElseIf`/
/// `Else` → [`Conditional`](SyntaxKind::Conditional); `For`/`VirtualFor` →
/// [`ForLoop`](SyntaxKind::ForLoop); everything else → [`Element`](SyntaxKind::Element).
fn classify_element_kind(name: &str) -> SyntaxKind {
    match name {
        "If" | "ElseIf" | "Else" => SyntaxKind::Conditional,
        "For" | "VirtualFor" => SyntaxKind::ForLoop,
        _ => SyntaxKind::Element,
    }
}

/// Materializes a balanced [`ParseEvent`] stream into a [`SyntaxNode`] tree and
/// its collected [`ParseError`]s.
///
/// An explicit stack of in-progress nodes is maintained:
/// [`StartNode`](ParseEvent::StartNode) pushes a node,
/// [`Token`](ParseEvent::Token) attaches to the top node,
/// [`FinishNode`](ParseEvent::FinishNode) pops the top and attaches it to the
/// new top (or, for the outermost node, becomes the root), and
/// [`Error`](ParseEvent::Error) is collected. Any unbalanced remainder is folded
/// defensively so a root is always produced.
fn build_tree(events: Vec<ParseEvent>) -> (SyntaxNode, Vec<ParseError>) {
    let mut stack: Vec<SyntaxNode> = Vec::new();
    let mut root: Option<SyntaxNode> = None;
    let mut errors: Vec<ParseError> = Vec::new();

    for event in events {
        match event {
            ParseEvent::StartNode(kind) => stack.push(SyntaxNode::new(kind)),
            ParseEvent::Token(token) => {
                if let Some(top) = stack.last_mut() {
                    top.push_token(token);
                }
            }
            ParseEvent::FinishNode => attach_or_root(&mut stack, &mut root),
            ParseEvent::Error(error) => errors.push(error),
        }
    }

    // Defensive: fold any nodes left open by an unbalanced stream.
    while !stack.is_empty() {
        attach_or_root(&mut stack, &mut root);
    }

    let root = root.unwrap_or_else(|| SyntaxNode::new(SyntaxKind::Document));
    (root, errors)
}

/// Pops the top node and attaches it to its parent, or records it as the root
/// when the stack becomes empty.
fn attach_or_root(stack: &mut Vec<SyntaxNode>, root: &mut Option<SyntaxNode>) {
    if let Some(node) = stack.pop() {
        match stack.last_mut() {
            Some(parent) => parent.push_node(node),
            None => *root = Some(node),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::SyntaxElement;

    /// Parses `source` and asserts the tree round-trips losslessly.
    fn parse_lossless(source: &str) -> ParseResult {
        let result = Parser::new().parse(source);
        assert_eq!(
            result.tree().text(),
            source,
            "lossless invariant violated for {source:?}"
        );
        result
    }

    /// Returns the immediate child nodes of the document root of `kind`.
    fn root_nodes_of<'a>(
        result: &'a ParseResult,
        kind: SyntaxKind,
    ) -> Vec<&'a SyntaxNode> {
        result
            .tree()
            .root()
            .child_nodes()
            .filter(|n| n.kind() == kind)
            .collect()
    }

    /// Recursively finds the first descendant node of the given `kind`.
    fn find_node(node: &SyntaxNode, kind: SyntaxKind) -> Option<&SyntaxNode> {
        for child in node.children() {
            if let SyntaxElement::Node(n) = child {
                if n.kind() == kind {
                    return Some(n);
                }
                if let Some(found) = find_node(n, kind) {
                    return Some(found);
                }
            }
        }
        None
    }

    #[test]
    fn parses_a_simple_element_losslessly() {
        let src = "<Button label=\"Save\">Hi</Button>";
        let result = parse_lossless(src);
        assert!(result.is_ok(), "errors: {:?}", result.errors());

        let elements = root_nodes_of(&result, SyntaxKind::Element);
        assert_eq!(elements.len(), 1);
        let element = elements[0];

        // OpenTag holds the `Button` ident and an Attribute.
        let open = find_node(element, SyntaxKind::OpenTag).expect("OpenTag");
        assert!(open.tokens().any(|t| t.text() == "Button"));
        assert!(open.child_nodes().any(|n| n.kind() == SyntaxKind::Attribute));

        // Element also has a TextNode and a CloseTag.
        assert!(element
            .child_nodes()
            .any(|n| n.kind() == SyntaxKind::TextNode));
        assert!(element
            .child_nodes()
            .any(|n| n.kind() == SyntaxKind::CloseTag));
    }

    #[test]
    fn parses_a_self_closing_element() {
        let src = "<Icon name=\"x\"/>";
        let result = parse_lossless(src);
        assert!(result.is_ok(), "errors: {:?}", result.errors());

        let elements = root_nodes_of(&result, SyntaxKind::Element);
        assert_eq!(elements.len(), 1);
        // No close tag for a self-closing element.
        assert!(find_node(elements[0], SyntaxKind::CloseTag).is_none());
    }

    #[test]
    fn interpolation_yields_an_expression_node() {
        let src = "<T>{count}</T>";
        let result = parse_lossless(src);
        assert!(result.is_ok(), "errors: {:?}", result.errors());

        let element = root_nodes_of(&result, SyntaxKind::Element)[0];
        let expr = find_node(element, SyntaxKind::Expression).expect("Expression");
        assert_eq!(expr.text(), "{count}");
    }

    #[test]
    fn if_element_is_classified_as_conditional() {
        // `<If ...>` is parsed as an Element node whose kind is `Conditional`.
        let src = "<If cond={open}><A/></If>";
        let result = parse_lossless(src);
        assert!(result.is_ok(), "errors: {:?}", result.errors());

        let conditionals = root_nodes_of(&result, SyntaxKind::Conditional);
        assert_eq!(conditionals.len(), 1);
        // The nested `<A/>` is a plain Element inside the conditional.
        assert!(find_node(conditionals[0], SyntaxKind::Element).is_some());
    }

    #[test]
    fn for_element_is_classified_as_for_loop() {
        let src = "<For each={items}><Row/></For>";
        let result = parse_lossless(src);
        assert!(result.is_ok(), "errors: {:?}", result.errors());
        assert_eq!(root_nodes_of(&result, SyntaxKind::ForLoop).len(), 1);
    }

    #[test]
    fn malformed_missing_close_recovers_losslessly() {
        let src = "<Button>oops";
        let result = parse_lossless(src);
        assert!(!result.is_ok(), "expected a recorded error");
        assert!(result
            .errors()
            .iter()
            .any(|e| e.kind() == ParseErrorKind::ExpectedClose));
    }

    #[test]
    fn mismatched_close_tag_is_reported_but_lossless() {
        let src = "<A>x</B>";
        let result = parse_lossless(src);
        assert!(result
            .errors()
            .iter()
            .any(|e| e.kind() == ParseErrorKind::MismatchedCloseTag));
    }

    #[test]
    fn unterminated_expression_is_reported_but_lossless() {
        let src = "<T>{count</T>";
        let result = parse_lossless(src);
        assert!(result
            .errors()
            .iter()
            .any(|e| e.kind() == ParseErrorKind::UnterminatedExpression));
    }

    #[test]
    fn empty_source_yields_an_empty_document() {
        let result = parse_lossless("");
        assert!(result.is_ok());
        assert_eq!(result.tree().root().kind(), SyntaxKind::Document);
    }

    #[test]
    fn plain_text_and_whitespace_round_trip() {
        let result = parse_lossless("  hello world  ");
        assert!(result.is_ok());
    }

    #[test]
    fn stray_close_tag_at_top_level_does_not_hang() {
        let result = parse_lossless("</Oops>");
        assert!(!result.is_ok());
    }

    #[test]
    fn nested_elements_round_trip() {
        let src = "<Col>\n  <Row a=\"1\">{x}</Row>\n  <Icon name=\"y\"/>\n</Col>";
        let result = parse_lossless(src);
        assert!(result.is_ok(), "errors: {:?}", result.errors());
    }
}
