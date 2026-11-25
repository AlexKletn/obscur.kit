//! The main-axis direction of a flex container.

#[doc = include_str!("../../docs/api/model/flex_direction.doc/FlexDirection.md")]
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
    #[doc = include_str!("../../docs/api/model/flex_direction.doc/is_row.md")]
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
