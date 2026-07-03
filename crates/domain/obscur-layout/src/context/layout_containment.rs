//! Whether a layout context contains layout/size effects at its boundary.

/// Declares whether a [`LayoutContext`](super::LayoutContext) *contains* layout
/// and/or size effects so that changes inside it do not cross its boundary
/// (spec §4.3; analogous to CSS `contain`).
///
/// A containment boundary stops **upward** invalidation propagation: when a
/// context contains layout, changes to its internal layout cannot escape to
/// affect ancestors, so incremental recalculation need not propagate past it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayoutContainment {
    /// Whether internal layout changes are contained (cannot escape the boundary).
    contains_layout: bool,
    /// Whether the node's own size is contained (independent of its contents).
    contains_size: bool,
}

impl LayoutContainment {
    /// No containment: neither layout nor size effects are contained.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            contains_layout: false,
            contains_size: false,
        }
    }

    /// Contains layout effects only.
    #[must_use]
    pub const fn layout() -> Self {
        Self {
            contains_layout: true,
            contains_size: false,
        }
    }

    /// Contains size effects only.
    #[must_use]
    pub const fn size() -> Self {
        Self {
            contains_layout: false,
            contains_size: true,
        }
    }

    /// Contains both layout and size effects.
    #[must_use]
    pub const fn layout_and_size() -> Self {
        Self {
            contains_layout: true,
            contains_size: true,
        }
    }

    /// Whether internal layout changes are contained.
    #[must_use]
    pub const fn contains_layout(self) -> bool {
        self.contains_layout
    }

    /// Whether the node's own size is contained.
    #[must_use]
    pub const fn contains_size(self) -> bool {
        self.contains_size
    }

    /// Whether this containment forms a boundary that stops upward propagation.
    ///
    /// True when layout is contained: inner layout changes cannot escape, so
    /// invalidation does not need to propagate to ancestors past this context.
    #[must_use]
    pub const fn is_boundary(self) -> bool {
        self.contains_layout
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutContainment;

    #[test]
    fn none_contains_nothing_and_is_not_boundary() {
        let c = LayoutContainment::none();
        assert!(!c.contains_layout());
        assert!(!c.contains_size());
        assert!(!c.is_boundary());
    }

    #[test]
    fn layout_is_boundary() {
        let c = LayoutContainment::layout();
        assert!(c.contains_layout());
        assert!(!c.contains_size());
        assert!(c.is_boundary());
    }

    #[test]
    fn size_only_is_not_boundary() {
        let c = LayoutContainment::size();
        assert!(!c.contains_layout());
        assert!(c.contains_size());
        assert!(!c.is_boundary());
    }

    #[test]
    fn layout_and_size_contains_both_and_is_boundary() {
        let c = LayoutContainment::layout_and_size();
        assert!(c.contains_layout());
        assert!(c.contains_size());
        assert!(c.is_boundary());
    }
}
