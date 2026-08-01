#![doc = include_str!("mask_kind.doc/_module.md")]

#[doc = include_str!("mask_kind.doc/MaskKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaskKind {
    #[doc = include_str!("mask_kind.doc/Alpha.md")]
    Alpha,
    #[doc = include_str!("mask_kind.doc/Luminance.md")]
    Luminance,
}

impl MaskKind {
    #[doc = include_str!("mask_kind.doc/is_luminance.md")]
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
