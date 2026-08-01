#![doc = include_str!("navigation_unit.doc/_module.md")]

#[doc = include_str!("navigation_unit.doc/NavigationUnit.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavigationUnit {
    #[doc = include_str!("navigation_unit.doc/Grapheme.md")]
    Grapheme,
    #[doc = include_str!("navigation_unit.doc/Word.md")]
    Word,
    #[doc = include_str!("navigation_unit.doc/Line.md")]
    Line,
    #[doc = include_str!("navigation_unit.doc/LineBoundary.md")]
    LineBoundary,
    #[doc = include_str!("navigation_unit.doc/Paragraph.md")]
    Paragraph,
    #[doc = include_str!("navigation_unit.doc/Document.md")]
    Document,
}

impl NavigationUnit {
    #[doc = include_str!("navigation_unit.doc/is_boundary.md")]
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
