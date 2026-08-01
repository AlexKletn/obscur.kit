#![doc = include_str!("invalidation_reason.doc/_module.md")]

use crate::invalidation::InvalidationKind;

#[doc = include_str!("invalidation_reason.doc/InvalidationReason.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvalidationReason {
    #[doc = include_str!("invalidation_reason.doc/StyleChanged.md")]
    StyleChanged,
    #[doc = include_str!("invalidation_reason.doc/InlineStyleChanged.md")]
    InlineStyleChanged,
    #[doc = include_str!("invalidation_reason.doc/PseudoStateChanged.md")]
    PseudoStateChanged,
    #[doc = include_str!("invalidation_reason.doc/CustomPropertyChanged.md")]
    CustomPropertyChanged,
    #[doc = include_str!("invalidation_reason.doc/LayoutChanged.md")]
    LayoutChanged,
    #[doc = include_str!("invalidation_reason.doc/TreeChanged.md")]
    TreeChanged,
    #[doc = include_str!("invalidation_reason.doc/ContentChanged.md")]
    ContentChanged,
    #[doc = include_str!("invalidation_reason.doc/EnvironmentChanged.md")]
    EnvironmentChanged,
}

impl InvalidationReason {
    #[doc = include_str!("invalidation_reason.doc/kind.md")]
    #[must_use]
    pub fn kind(self) -> InvalidationKind {
        match self {
            InvalidationReason::StyleChanged
            | InvalidationReason::InlineStyleChanged
            | InvalidationReason::PseudoStateChanged
            | InvalidationReason::CustomPropertyChanged => InvalidationKind::Style,
            InvalidationReason::LayoutChanged => InvalidationKind::Layout,
            InvalidationReason::TreeChanged => InvalidationKind::Tree,
            InvalidationReason::ContentChanged => InvalidationKind::Text,
            InvalidationReason::EnvironmentChanged => InvalidationKind::Environment,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_causes_map_to_style_kind() {
        for r in [
            InvalidationReason::StyleChanged,
            InvalidationReason::InlineStyleChanged,
            InvalidationReason::PseudoStateChanged,
            InvalidationReason::CustomPropertyChanged,
        ] {
            assert_eq!(r.kind(), InvalidationKind::Style);
        }
    }

    #[test]
    fn structural_and_content_causes_map_to_their_kinds() {
        assert_eq!(
            InvalidationReason::TreeChanged.kind(),
            InvalidationKind::Tree
        );
        assert_eq!(
            InvalidationReason::ContentChanged.kind(),
            InvalidationKind::Text
        );
        assert_eq!(
            InvalidationReason::EnvironmentChanged.kind(),
            InvalidationKind::Environment
        );
    }
}
