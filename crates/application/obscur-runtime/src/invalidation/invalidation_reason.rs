//! The runtime-level cause of a change (Obscur Kit 2 specs §4.3).

use crate::invalidation::InvalidationKind;

/// Why a change happened — the runtime-level cause — which maps to an
/// [`InvalidationKind`] and, via the router, to the domain invalidation reasons.
///
/// Each variant records the semantic cause of an invalidation as reported to the
/// runtime; [`kind`](InvalidationReason::kind) classifies it into the shared
/// [`InvalidationKind`] depth used to flag dirty pipeline phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvalidationReason {
    /// A node's style / declarations changed (e.g. a stylesheet rule now matches
    /// differently). Maps to [`InvalidationKind::Style`].
    StyleChanged,
    /// A node's inline style attribute changed. Maps to [`InvalidationKind::Style`].
    InlineStyleChanged,
    /// A node's pseudo-state changed (hover/focus/etc.). Maps to
    /// [`InvalidationKind::Style`].
    PseudoStateChanged,
    /// A custom property (CSS variable) affecting a node changed. Maps to
    /// [`InvalidationKind::Style`].
    CustomPropertyChanged,
    /// A node's box geometry must be recomputed without its style having changed.
    /// Maps to [`InvalidationKind::Layout`].
    ///
    /// The cause a caller states when it knows the *result* of a style rather than
    /// the style itself — it asks for a relayout and nothing more, so the style
    /// phase is not dirtied. [`UiOperation::InvalidateLayout`] is exactly that
    /// statement, and this is what it means.
    ///
    /// [`UiOperation::InvalidateLayout`]: crate::transaction::UiOperation::InvalidateLayout
    LayoutChanged,
    /// The node tree changed (children added/removed). Maps to
    /// [`InvalidationKind::Tree`].
    TreeChanged,
    /// A node's text / intrinsic content changed. Maps to
    /// [`InvalidationKind::Text`].
    ContentChanged,
    /// The window/environment changed (viewport, DPI). Maps to
    /// [`InvalidationKind::Environment`].
    EnvironmentChanged,
}

impl InvalidationReason {
    /// Classifies this cause into its [`InvalidationKind`] depth.
    ///
    /// The style-affecting causes ([`StyleChanged`](InvalidationReason::StyleChanged),
    /// [`InlineStyleChanged`](InvalidationReason::InlineStyleChanged),
    /// [`PseudoStateChanged`](InvalidationReason::PseudoStateChanged),
    /// [`CustomPropertyChanged`](InvalidationReason::CustomPropertyChanged)) all map
    /// to [`InvalidationKind::Style`]; the remaining causes map to their matching
    /// depths.
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
