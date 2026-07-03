//! How a mask's coverage is derived from its source content.

/// Whether a [`Mask`](super::Mask)'s coverage is taken from the source's alpha
/// channel or from its luminance (spec §5.8).
///
/// This selects the interpretation of the mask's source content; it carries no
/// coverage data itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaskKind {
    /// Coverage comes from the source's alpha channel.
    Alpha,
    /// Coverage comes from the source's luminance (perceived brightness).
    Luminance,
}

impl MaskKind {
    /// Returns whether coverage is derived from luminance rather than alpha.
    #[must_use]
    pub const fn is_luminance(self) -> bool {
        matches!(self, Self::Luminance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_luminance_reflects_variant() {
        assert!(MaskKind::Luminance.is_luminance());
        assert!(!MaskKind::Alpha.is_luminance());
    }

    #[test]
    fn is_copy_eq_hash() {
        use std::collections::HashSet;
        let a = MaskKind::Alpha;
        let copied = a;
        assert_eq!(a, copied);
        let mut set = HashSet::new();
        set.insert(MaskKind::Alpha);
        set.insert(MaskKind::Luminance);
        assert_eq!(set.len(), 2);
    }
}
