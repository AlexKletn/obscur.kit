//! `Parser`: an error-tolerant recursive-descent parser for `.okui` source
//! (spec §2.9) that always yields a lossless [`SyntaxTree`].

mod parse_walker;

use parse_walker::ParseWalker;

use crate::lexer::Lexer;
use crate::parser::{ParseError, ParseEvent, ParseResult};
use crate::syntax::{SyntaxKind, SyntaxNode, SyntaxTree};

#[doc = include_str!("../../docs/api/parser/parser.doc/Parser.md")]
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

    #[doc = include_str!("../../docs/api/parser/parser.doc/parse.md")]
    #[must_use]
    pub fn parse(&self, source: &str) -> ParseResult {
        let tokens = Lexer::new().tokenize(source);
        let (root, errors) = build_tree(ParseWalker::parse(&tokens));
        ParseResult::new(SyntaxTree::new(root), errors)
    }
}

#[doc = include_str!("../../docs/api/parser/parser.doc/build_tree.md")]
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
    use crate::parser::ParseErrorKind;
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
