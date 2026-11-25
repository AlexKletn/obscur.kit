//! The Porter-Duff compositing operator for a layer.

#[doc = include_str!("../../docs/api/layer/composition_mode.doc/CompositionMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompositionMode {
    /// Source over destination: normal alpha compositing (the default).
    SrcOver,
    /// Source replaces destination, ignoring the destination entirely.
    Src,
    /// Destination over source: the source shows only where the destination is
    /// transparent.
    DstOver,
    /// Clears the destination to transparent, drawing neither operand's color.
    Clear,
}

impl CompositionMode {
    /// Returns whether this is [`CompositionMode::SrcOver`] (normal alpha
    /// compositing).
    #[must_use]
    pub const fn is_src_over(self) -> bool {
        matches!(self, Self::SrcOver)
    }
}

impl Default for CompositionMode {
    /// The default operator is [`CompositionMode::SrcOver`].
    fn default() -> Self {
        Self::SrcOver
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_src_over() {
        assert_eq!(CompositionMode::default(), CompositionMode::SrcOver);
        assert!(CompositionMode::default().is_src_over());
    }

    #[test]
    fn non_default_variants_are_not_src_over() {
        assert!(!CompositionMode::Src.is_src_over());
        assert!(!CompositionMode::DstOver.is_src_over());
        assert!(!CompositionMode::Clear.is_src_over());
    }

    #[test]
    fn is_copy_eq_hash() {
        use std::collections::HashSet;
        let m = CompositionMode::Src;
        let copied = m;
        assert_eq!(m, copied);
        let mut set = HashSet::new();
        set.insert(CompositionMode::SrcOver);
        set.insert(CompositionMode::Clear);
        assert_eq!(set.len(), 2);
    }
}
