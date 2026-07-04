//! `SyntaxNode`: an interior node of the lossless syntax tree, plus the
//! [`SyntaxElement`] child enum.

use crate::syntax::{SyntaxKind, SyntaxToken};

/// A child of a [`SyntaxNode`]: either an interior node or a leaf token.
#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxElement {
    /// An interior node.
    Node(SyntaxNode),
    /// A leaf token.
    Token(SyntaxToken),
}

impl SyntaxElement {
    /// Returns the exact source slice covered by this child.
    #[must_use]
    pub fn text(&self) -> String {
        match self {
            Self::Node(node) => node.text(),
            Self::Token(token) => token.text().to_owned(),
        }
    }

    /// Returns the byte offset of this child's first token, if any.
    #[must_use]
    pub fn offset(&self) -> Option<u32> {
        match self {
            Self::Node(node) => node.offset(),
            Self::Token(token) => Some(token.offset()),
        }
    }
}

/// An interior node of the lossless syntax tree (spec §2.8).
///
/// A node is labelled with a node-valued [`SyntaxKind`] and owns an ordered
/// list of [`SyntaxElement`] children (nodes and tokens interleaved).
///
/// ## Lossless invariant
///
/// [`SyntaxNode::text`] reconstructs the exact source substring the node
/// covers by concatenating, in order, the text of every descendant token. For
/// the tree's root this reproduces the whole source byte-for-byte.
#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxNode {
    kind: SyntaxKind,
    children: Vec<SyntaxElement>,
}

impl SyntaxNode {
    /// Creates an empty node of the given (node-valued) `kind`.
    #[must_use]
    pub fn new(kind: SyntaxKind) -> Self {
        Self {
            kind,
            children: Vec::new(),
        }
    }

    /// Builder: appends `child` and returns `self`.
    #[must_use]
    pub fn with_child(mut self, child: SyntaxElement) -> Self {
        self.children.push(child);
        self
    }

    /// Appends a child element.
    pub fn push(&mut self, child: SyntaxElement) {
        self.children.push(child);
    }

    /// Appends a child node.
    pub fn push_node(&mut self, node: SyntaxNode) {
        self.children.push(SyntaxElement::Node(node));
    }

    /// Appends a child token.
    pub fn push_token(&mut self, token: SyntaxToken) {
        self.children.push(SyntaxElement::Token(token));
    }

    /// Returns the node's kind.
    #[must_use]
    pub const fn kind(&self) -> SyntaxKind {
        self.kind
    }

    /// Returns the node's children in order.
    #[must_use]
    pub fn children(&self) -> &[SyntaxElement] {
        &self.children
    }

    /// Iterates the node's immediate child nodes.
    pub fn child_nodes(&self) -> impl Iterator<Item = &SyntaxNode> {
        self.children.iter().filter_map(|child| match child {
            SyntaxElement::Node(node) => Some(node),
            SyntaxElement::Token(_) => None,
        })
    }

    /// Iterates the node's immediate child tokens.
    pub fn tokens(&self) -> impl Iterator<Item = &SyntaxToken> {
        self.children.iter().filter_map(|child| match child {
            SyntaxElement::Token(token) => Some(token),
            SyntaxElement::Node(_) => None,
        })
    }

    /// Reconstructs the source substring this node covers, by concatenating the
    /// text of every descendant token in order (recursively).
    #[must_use]
    pub fn text(&self) -> String {
        let mut out = String::new();
        self.write_text(&mut out);
        out
    }

    fn write_text(&self, out: &mut String) {
        for child in &self.children {
            match child {
                SyntaxElement::Node(node) => node.write_text(out),
                SyntaxElement::Token(token) => out.push_str(token.text()),
            }
        }
    }

    /// Returns this node's first descendant token, if any.
    #[must_use]
    pub fn first_token(&self) -> Option<&SyntaxToken> {
        for child in &self.children {
            match child {
                SyntaxElement::Token(token) => return Some(token),
                SyntaxElement::Node(node) => {
                    if let Some(token) = node.first_token() {
                        return Some(token);
                    }
                }
            }
        }
        None
    }

    /// Returns the byte offset of this node's first descendant token, if any.
    #[must_use]
    pub fn offset(&self) -> Option<u32> {
        self.first_token().map(SyntaxToken::offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(kind: SyntaxKind, text: &str, offset: u32) -> SyntaxToken {
        SyntaxToken::new(kind, text, offset)
    }

    #[test]
    fn text_concatenates_child_tokens() {
        let node = SyntaxNode::new(SyntaxKind::OpenTag)
            .with_child(SyntaxElement::Token(tok(SyntaxKind::LAngle, "<", 0)))
            .with_child(SyntaxElement::Token(tok(SyntaxKind::Ident, "Button", 1)));
        assert_eq!(node.text(), "<Button");
        assert_eq!(node.offset(), Some(0));
    }

    #[test]
    fn text_reconstructs_recursively() {
        let inner = SyntaxNode::new(SyntaxKind::Expression)
            .with_child(SyntaxElement::Token(tok(SyntaxKind::LBrace, "{", 7)))
            .with_child(SyntaxElement::Token(tok(SyntaxKind::Ident, "count", 8)))
            .with_child(SyntaxElement::Token(tok(SyntaxKind::RBrace, "}", 13)));
        let outer = SyntaxNode::new(SyntaxKind::Element)
            .with_child(SyntaxElement::Token(tok(SyntaxKind::Text, "hi ", 4)))
            .with_child(SyntaxElement::Node(inner));
        assert_eq!(outer.text(), "hi {count}");
        assert_eq!(outer.offset(), Some(4));
    }

    #[test]
    fn child_nodes_and_tokens_iterators_separate_kinds() {
        let mut node = SyntaxNode::new(SyntaxKind::Element);
        node.push_token(tok(SyntaxKind::LAngle, "<", 0));
        node.push_node(SyntaxNode::new(SyntaxKind::OpenTag));
        node.push_token(tok(SyntaxKind::RAngle, ">", 1));

        assert_eq!(node.tokens().count(), 2);
        assert_eq!(node.child_nodes().count(), 1);
        assert_eq!(node.children().len(), 3);
        assert_eq!(node.kind(), SyntaxKind::Element);
    }

    #[test]
    fn empty_node_has_no_offset() {
        let node = SyntaxNode::new(SyntaxKind::Document);
        assert_eq!(node.offset(), None);
        assert!(node.first_token().is_none());
        assert_eq!(node.text(), "");
    }
}
