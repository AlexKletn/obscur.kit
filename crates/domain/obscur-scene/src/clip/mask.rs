//! A mask clip: coverage derived from separate source content.

use super::MaskKind;

#[doc = include_str!("../../docs/api/clip/mask.doc/Mask.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mask {
    kind: MaskKind,
}

impl Mask {
    /// Creates a mask whose coverage comes from the source's alpha channel.
    #[must_use]
    pub const fn alpha() -> Self {
        Self {
            kind: MaskKind::Alpha,
        }
    }

    /// Creates a mask whose coverage comes from the source's luminance.
    #[must_use]
    pub const fn luminance() -> Self {
        Self {
            kind: MaskKind::Luminance,
        }
    }

    /// The mask kind: how the source's coverage is interpreted.
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
