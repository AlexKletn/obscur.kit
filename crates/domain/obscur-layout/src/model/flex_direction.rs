//! The main-axis direction of a flex container.

/// The direction along which a flex container lays out its items (spec §4.1).
///
/// The direction selects the main axis (row vs. column) and whether items are
/// placed in forward or reverse order along it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlexDirection {
    /// Main axis is horizontal, items placed start-to-end (the default).
    #[default]
    Row,
    /// Main axis is vertical, items placed start-to-end.
    Column,
    /// Main axis is horizontal, items placed end-to-start.
    RowReverse,
    /// Main axis is vertical, items placed end-to-start.
    ColumnReverse,
}

impl FlexDirection {
    /// Returns `true` if the main axis is horizontal ([`Row`] or [`RowReverse`]).
    ///
    /// [`Row`]: FlexDirection::Row
    /// [`RowReverse`]: FlexDirection::RowReverse
    #[must_use]
    pub const fn is_row(self) -> bool {
        matches!(self, Self::Row | Self::RowReverse)
    }

    /// Returns `true` if items are placed in reverse order along the main axis.
    #[must_use]
    pub const fn is_reverse(self) -> bool {
        matches!(self, Self::RowReverse | Self::ColumnReverse)
    }
}

#[cfg(test)]
mod tests {
    use super::FlexDirection;

    #[test]
    fn default_is_row() {
        assert_eq!(FlexDirection::default(), FlexDirection::Row);
    }

    #[test]
    fn is_row_classification() {
        assert!(FlexDirection::Row.is_row());
        assert!(FlexDirection::RowReverse.is_row());
        assert!(!FlexDirection::Column.is_row());
        assert!(!FlexDirection::ColumnReverse.is_row());
    }

    #[test]
    fn is_reverse_classification() {
        assert!(FlexDirection::RowReverse.is_reverse());
        assert!(FlexDirection::ColumnReverse.is_reverse());
        assert!(!FlexDirection::Row.is_reverse());
        assert!(!FlexDirection::Column.is_reverse());
    }
}
