//! What happens to text that does not fit its available width (spec §5.10).

#[doc = include_str!("../../docs/api/layout/text_overflow.doc/TextOverflow.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextOverflow {
    /// Let overflowing text spill visibly past the available width.
    Visible,
    /// Hard-clip overflowing text at the boundary. The default.
    #[default]
    Clip,
    /// Truncate overflowing text and mark the cut with an ellipsis.
    Ellipsis,
}

impl TextOverflow {
    /// Returns whether this policy truncates overflowing text — true for
    /// [`Clip`](Self::Clip) and [`Ellipsis`](Self::Ellipsis), false for
    /// [`Visible`](Self::Visible) (spec §5.10).
    #[must_use]
    pub const fn is_truncating(&self) -> bool {
        matches!(self, Self::Clip | Self::Ellipsis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_clip() {
        assert_eq!(TextOverflow::default(), TextOverflow::Clip);
    }

    #[test]
    fn is_truncating_covers_clip_and_ellipsis() {
        assert!(!TextOverflow::Visible.is_truncating());
        assert!(TextOverflow::Clip.is_truncating());
        assert!(TextOverflow::Ellipsis.is_truncating());
    }
}
