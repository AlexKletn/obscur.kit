//! The granularity of caret movement in the editing engine (spec §5.11).

/// The granularity of a caret movement, i.e. how far one navigation step travels
/// (spec §5.11 editing engine — word/line/document navigation, Home/End).
///
/// A [`NavigationUnit`] answers "by how much?" for a move or shift-extend: it
/// pairs with a direction (and an *extend* flag) in
/// [`EditCommand::Move`](crate::editing::EditCommand::Move) to describe every
/// caret motion a keymap can bind. It carries no direction of its own — moving
/// left or right by a word both use [`Word`](Self::Word).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavigationUnit {
    /// One extended grapheme cluster — the Left/Right arrow keys.
    Grapheme,
    /// One word — Ctrl+Left / Ctrl+Right (Alt+Left / Alt+Right on macOS).
    Word,
    /// One visual (wrapped) or logical line in the move direction — the Up/Down
    /// arrow keys.
    Line,
    /// The nearest line boundary — Home (start) / End (end) within the current
    /// visual line.
    LineBoundary,
    /// One paragraph — Ctrl+Up / Ctrl+Down (Alt+Up / Alt+Down on macOS).
    Paragraph,
    /// The start or end of the whole document — Ctrl+Home / Ctrl+End.
    Document,
}

impl NavigationUnit {
    /// Returns whether this unit snaps the caret to a fixed edge rather than
    /// advancing it a countable amount.
    ///
    /// True for [`LineBoundary`](Self::LineBoundary) (Home/End) and
    /// [`Document`](Self::Document) (Ctrl+Home/End): both jump to an edge, so a
    /// repeated move in the same direction is idempotent, whereas the other
    /// units keep advancing.
    #[must_use]
    pub const fn is_boundary(&self) -> bool {
        matches!(self, Self::LineBoundary | Self::Document)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_units() {
        assert!(NavigationUnit::LineBoundary.is_boundary());
        assert!(NavigationUnit::Document.is_boundary());
    }

    #[test]
    fn non_boundary_units() {
        assert!(!NavigationUnit::Grapheme.is_boundary());
        assert!(!NavigationUnit::Word.is_boundary());
        assert!(!NavigationUnit::Line.is_boundary());
        assert!(!NavigationUnit::Paragraph.is_boundary());
    }
}
