#![doc = include_str!("text_overflow.doc/_module.md")]

#[doc = include_str!("text_overflow.doc/TextOverflow.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextOverflow {
    #[doc = include_str!("text_overflow.doc/Visible.md")]
    Visible,
    #[doc = include_str!("text_overflow.doc/Clip.md")]
    #[default]
    Clip,
    #[doc = include_str!("text_overflow.doc/Ellipsis.md")]
    Ellipsis,
}

impl TextOverflow {
    #[doc = include_str!("text_overflow.doc/is_truncating.md")]
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
