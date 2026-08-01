#![doc = include_str!("composition_mode.doc/_module.md")]

#[doc = include_str!("composition_mode.doc/CompositionMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompositionMode {
    #[doc = include_str!("composition_mode.doc/SrcOver.md")]
    SrcOver,
    #[doc = include_str!("composition_mode.doc/Src.md")]
    Src,
    #[doc = include_str!("composition_mode.doc/DstOver.md")]
    DstOver,
    #[doc = include_str!("composition_mode.doc/Clear.md")]
    Clear,
}

impl CompositionMode {
    #[doc = include_str!("composition_mode.doc/is_src_over.md")]
    #[must_use]
    pub const fn is_src_over(self) -> bool {
        matches!(self, Self::SrcOver)
    }
}

impl Default for CompositionMode {
    #[doc = include_str!("composition_mode.doc/default.md")]
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
