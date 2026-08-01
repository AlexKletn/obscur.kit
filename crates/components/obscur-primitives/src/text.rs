#![doc = include_str!("text.doc/_module.md")]

use obscur_view::view::{View, ViewNode};

#[doc = include_str!("text.doc/Text.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    content: Box<str>,
}

impl Text {
    #[doc = include_str!("text.doc/new.md")]
    #[must_use]
    pub fn new(content: impl Into<Box<str>>) -> Self {
        Self {
            content: content.into(),
        }
    }

    #[doc = include_str!("text.doc/content.md")]
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    #[doc = include_str!("text.doc/into_view_node.md")]
    #[must_use]
    pub fn into_view_node(self) -> ViewNode {
        ViewNode::text(self.content)
    }

    #[doc = include_str!("text.doc/view.md")]
    #[must_use]
    pub fn view(self) -> View {
        View::single(self.into_view_node())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_content() {
        let t = Text::new("hello");
        assert_eq!(t.content(), "hello");
    }

    #[test]
    fn lowers_to_text_node() {
        let node = Text::new("hello").into_view_node();
        if let ViewNode::Text { content, .. } = node {
            assert_eq!(&*content, "hello");
        } else {
            panic!("expected text node");
        }
    }

    #[test]
    fn view_wraps_single_node() {
        let view = Text::new("hi").view();
        assert_eq!(view.root().len(), 1);
    }
}
