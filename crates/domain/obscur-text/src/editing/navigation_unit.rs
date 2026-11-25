//! The granularity of caret movement in the editing engine (spec §5.11).

#[doc = include_str!("../../docs/api/editing/navigation_unit.doc/NavigationUnit.md")]
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
    #[doc = include_str!("../../docs/api/editing/navigation_unit.doc/is_boundary.md")]
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
