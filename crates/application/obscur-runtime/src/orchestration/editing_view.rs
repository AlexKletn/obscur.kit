#![doc = include_str!("editing_view.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_text::selection::TextSelection;

#[doc = include_str!("editing_view.doc/EditingView.md")]
#[derive(Debug, Clone, Copy)]
pub struct EditingView<'a> {
    #[doc = include_str!("editing_view.doc/node.md")]
    pub node: NodeId,
    #[doc = include_str!("editing_view.doc/text.md")]
    pub text: &'a str,
    #[doc = include_str!("editing_view.doc/selection.md")]
    pub selection: TextSelection,
    #[doc = include_str!("editing_view.doc/preedit.md")]
    pub preedit: Option<PreeditView<'a>>,
    #[doc = include_str!("editing_view.doc/scroll_left.md")]
    pub scroll_left: f32,
}

#[doc = include_str!("editing_view.doc/PreeditView.md")]
#[derive(Debug, Clone, Copy)]
pub struct PreeditView<'a> {
    #[doc = include_str!("editing_view.doc/text_2.md")]
    pub text: &'a str,
    #[doc = include_str!("editing_view.doc/at.md")]
    pub at: usize,
    #[doc = include_str!("editing_view.doc/end.md")]
    pub end: usize,
    #[doc = include_str!("editing_view.doc/caret.md")]
    pub caret: usize,
}

impl<'a> PreeditView<'a> {
    #[doc = include_str!("editing_view.doc/new.md")]
    #[must_use]
    pub(crate) const fn new(text: &'a str, at: usize, end: usize, caret: usize) -> Self {
        Self {
            text,
            at,
            end,
            caret,
        }
    }
}

impl<'a> EditingView<'a> {
    #[doc = include_str!("editing_view.doc/new_2.md")]
    #[must_use]
    pub(crate) const fn new(node: NodeId, text: &'a str, selection: TextSelection) -> Self {
        Self {
            node,
            text,
            selection,
            preedit: None,
            scroll_left: 0.0,
        }
    }

    #[doc = include_str!("editing_view.doc/with_preedit.md")]
    #[must_use]
    pub(crate) const fn with_preedit(mut self, preedit: PreeditView<'a>) -> Self {
        self.preedit = Some(preedit);
        self
    }

    #[doc = include_str!("editing_view.doc/with_scroll_left.md")]
    #[must_use]
    pub(crate) const fn with_scroll_left(mut self, scroll_left: f32) -> Self {
        self.scroll_left = scroll_left;
        self
    }
}
