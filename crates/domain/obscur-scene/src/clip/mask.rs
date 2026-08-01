#![doc = include_str!("mask.doc/_module.md")]

use super::MaskKind;

#[doc = include_str!("mask.doc/Mask.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mask {
    kind: MaskKind,
}

impl Mask {
    #[doc = include_str!("mask.doc/alpha.md")]
    #[must_use]
    pub const fn alpha() -> Self {
        Self {
            kind: MaskKind::Alpha,
        }
    }

    #[doc = include_str!("mask.doc/luminance.md")]
    #[must_use]
    pub const fn luminance() -> Self {
        Self {
            kind: MaskKind::Luminance,
        }
    }

    #[doc = include_str!("mask.doc/kind.md")]
    #[must_use]
    pub const fn kind(self) -> MaskKind {
        self.kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_constructor_records_alpha_kind() {
        assert_eq!(Mask::alpha().kind(), MaskKind::Alpha);
    }

    #[test]
    fn luminance_constructor_records_luminance_kind() {
        assert_eq!(Mask::luminance().kind(), MaskKind::Luminance);
        assert!(Mask::luminance().kind().is_luminance());
    }

    #[test]
    fn is_copy_and_eq() {
        let m = Mask::alpha();
        let copied = m;
        assert_eq!(m, copied);
        assert_ne!(Mask::alpha(), Mask::luminance());
    }
}
