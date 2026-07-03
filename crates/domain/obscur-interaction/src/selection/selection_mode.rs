//! The [`SelectionMode`] cardinality governing how many items may be selected
//! (item-selection model).

/// How many items a [`Selection`](super::Selection) may hold at once.
///
/// This is the *item* selection model — selecting list rows, canvas objects, or
/// tree nodes — and is distinct from text-caret selection in `obscur-text`.
///
/// - [`None`](Self::None) disables selection entirely.
/// - [`Single`](Self::Single) allows at most one selected item.
/// - [`Multiple`](Self::Multiple) allows many selected items and supports
///   range (shift) extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelectionMode {
    /// Selection is disabled: no item can be selected.
    None,
    /// At most one item may be selected at a time.
    Single,
    /// Many items may be selected, including contiguous ranges.
    Multiple,
}

impl SelectionMode {
    /// Returns whether this mode allows more than one item to be selected
    /// (only [`Multiple`](Self::Multiple)).
    #[must_use]
    pub const fn allows_multiple(&self) -> bool {
        matches!(self, Self::Multiple)
    }

    /// Returns whether selection is enabled at all
    /// ([`Single`](Self::Single) or [`Multiple`](Self::Multiple)).
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        matches!(self, Self::Single | Self::Multiple)
    }
}

impl Default for SelectionMode {
    /// Defaults to [`Single`](Self::Single).
    fn default() -> Self {
        Self::Single
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_single() {
        assert_eq!(SelectionMode::default(), SelectionMode::Single);
    }

    #[test]
    fn allows_multiple_only_for_multiple() {
        assert!(SelectionMode::Multiple.allows_multiple());
        assert!(!SelectionMode::Single.allows_multiple());
        assert!(!SelectionMode::None.allows_multiple());
    }

    #[test]
    fn is_enabled_for_single_and_multiple() {
        assert!(SelectionMode::Single.is_enabled());
        assert!(SelectionMode::Multiple.is_enabled());
        assert!(!SelectionMode::None.is_enabled());
    }
}
