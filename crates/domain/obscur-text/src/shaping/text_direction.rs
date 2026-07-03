//! The base (paragraph) writing direction of a text run (spec §5.10).

/// The base — or paragraph — writing direction of a text run (spec §5.10).
///
/// This is the *base direction* only: it seeds bidirectional layout but does not
/// itself encode per-character bidi levels. Full Unicode Bidirectional Algorithm
/// level resolution — splitting mixed left-to-right and right-to-left text into
/// runs and reordering them — is an adapter concern that happens during
/// itemization, before an individual run reaches the shaper; by the time a run is
/// shaped its direction is already uniform and known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextDirection {
    /// Left-to-right base direction (e.g. Latin, Cyrillic); the default.
    #[default]
    LeftToRight,
    /// Right-to-left base direction (e.g. Arabic, Hebrew).
    RightToLeft,
}

impl TextDirection {
    /// Returns whether this is the [`RightToLeft`](Self::RightToLeft) direction.
    #[must_use]
    pub const fn is_rtl(&self) -> bool {
        matches!(self, Self::RightToLeft)
    }

    /// Returns whether this is the [`LeftToRight`](Self::LeftToRight) direction.
    #[must_use]
    pub const fn is_ltr(&self) -> bool {
        matches!(self, Self::LeftToRight)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_left_to_right() {
        assert_eq!(TextDirection::default(), TextDirection::LeftToRight);
    }

    #[test]
    fn direction_predicates() {
        assert!(TextDirection::LeftToRight.is_ltr());
        assert!(!TextDirection::LeftToRight.is_rtl());
        assert!(TextDirection::RightToLeft.is_rtl());
        assert!(!TextDirection::RightToLeft.is_ltr());
    }
}
