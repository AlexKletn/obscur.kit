#![doc = include_str!("parser.doc/_module.md")]

use crate::lexer::{Lexer, Token, TokenKind};
use crate::parser::{ParseError, ParseErrorKind, ParseEvent, ParseResult};
use crate::syntax::{SyntaxKind, SyntaxNode, SyntaxToken, SyntaxTree};

#[doc = include_str!("parser.doc/Parser.md")]
#[derive(Debug, Clone, Default)]
pub struct Parser {
    _private: (),
}

impl Parser {
    #[doc = include_str!("parser.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }

    #[doc = include_str!("parser.doc/parse.md")]
    #[must_use]
    pub fn parse(&self, source: &str) -> ParseResult {
        let tokens = Lexer::new().tokenize(source);
        let mut walker = Walker::new(&tokens);
        walker.parse_document();
        let (root, errors) = build_tree(walker.events);
        ParseResult::new(SyntaxTree::new(root), errors)
    }
}

#[doc = include_str!("parser.doc/Walker.md")]
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

    #[doc = include_str!("parser.doc/nth_kind.md")]
    fn nth_kind(&self, n: usize) -> TokenKind {
        self.tokens
            .get(self.pos + n)
            .map_or(TokenKind::Eof, Token::kind)
    }

    #[doc = include_str!("parser.doc/peek.md")]
    fn peek(&self) -> TokenKind {
        self.nth_kind(0)
    }

    #[doc = include_str!("parser.doc/nth_text.md")]
    fn nth_text(&self, n: usize) -> &'t str {
        self.tokens.get(self.pos + n).map_or("", Token::text)
    }

    #[doc = include_str!("parser.doc/offset.md")]
    fn offset(&self) -> u32 {
        self.tokens.get(self.pos).map_or(0, Token::offset)
    }

    #[doc = include_str!("parser.doc/at_end.md")]
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

    #[doc = include_str!("parser.doc/bump.md")]
    fn bump(&mut self) {
        if let Some(tok) = self.tokens.get(self.pos)
            && !tok.kind().is_eof()
        {
            self.events
                .push(ParseEvent::Token(SyntaxToken::from_lexer(tok)));
            self.pos += 1;
        }
    }

    #[doc = include_str!("parser.doc/bump_trivia.md")]
    fn bump_trivia(&mut self) {
        while self.peek() == TokenKind::Whitespace {
            self.bump();
        }
    }

    #[doc = include_str!("parser.doc/parse_document.md")]
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

    #[doc = include_str!("parser.doc/parse_element_or_stray.md")]
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

    #[doc = include_str!("parser.doc/parse_element.md")]
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

    #[doc = include_str!("parser.doc/parse_open_tag.md")]
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

    #[doc = include_str!("parser.doc/parse_attribute.md")]
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

    #[doc = include_str!("parser.doc/parse_attribute_value.md")]
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

    #[doc = include_str!("parser.doc/parse_expression.md")]
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

    #[doc = include_str!("parser.doc/parse_children.md")]
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

    #[doc = include_str!("parser.doc/parse_close_tag.md")]
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

    #[doc = include_str!("parser.doc/parse_text_node.md")]
    fn parse_text_node(&mut self) {
        self.start(SyntaxKind::TextNode);
        while !self.at_end() && self.peek() != TokenKind::LAngle && self.peek() != TokenKind::LBrace
        {
            self.bump();
        }
        self.finish();
    }
}

#[doc = include_str!("parser.doc/classify_element_kind.md")]
fn classify_element_kind(name: &str) -> SyntaxKind {
    match name {
        "If" | "ElseIf" | "Else" => SyntaxKind::Conditional,
        "For" | "VirtualFor" => SyntaxKind::ForLoop,
        _ => SyntaxKind::Element,
    }
}

#[doc = include_str!("parser.doc/build_tree.md")]
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

#[doc = include_str!("parser.doc/attach_or_root.md")]
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
    fn root_nodes_of<'a>(result: &'a ParseResult, kind: SyntaxKind) -> Vec<&'a SyntaxNode> {
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
        assert!(
            open.child_nodes()
                .any(|n| n.kind() == SyntaxKind::Attribute)
        );

        // Element also has a TextNode and a CloseTag.
        assert!(
            element
                .child_nodes()
                .any(|n| n.kind() == SyntaxKind::TextNode)
        );
        assert!(
            element
                .child_nodes()
                .any(|n| n.kind() == SyntaxKind::CloseTag)
        );
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
        assert!(
            result
                .errors()
                .iter()
                .any(|e| e.kind() == ParseErrorKind::ExpectedClose)
        );
    }

    #[test]
    fn mismatched_close_tag_is_reported_but_lossless() {
        let src = "<A>x</B>";
        let result = parse_lossless(src);
        assert!(
            result
                .errors()
                .iter()
                .any(|e| e.kind() == ParseErrorKind::MismatchedCloseTag)
        );
    }

    #[test]
    fn unterminated_expression_is_reported_but_lossless() {
        let src = "<T>{count</T>";
        let result = parse_lossless(src);
        assert!(
            result
                .errors()
                .iter()
                .any(|e| e.kind() == ParseErrorKind::UnterminatedExpression)
        );
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
