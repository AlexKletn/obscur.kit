#![doc = include_str!("flex_direction.doc/_module.md")]

#[doc = include_str!("flex_direction.doc/FlexDirection.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlexDirection {
    #[doc = include_str!("flex_direction.doc/Row.md")]
    #[default]
    Row,
    #[doc = include_str!("flex_direction.doc/Column.md")]
    Column,
    #[doc = include_str!("flex_direction.doc/RowReverse.md")]
    RowReverse,
    #[doc = include_str!("flex_direction.doc/ColumnReverse.md")]
    ColumnReverse,
}

impl FlexDirection {
    #[doc = include_str!("flex_direction.doc/is_row.md")]
    #[must_use]
    pub const fn is_row(self) -> bool {
        matches!(self, Self::Row | Self::RowReverse)
    }

    #[doc = include_str!("flex_direction.doc/is_reverse.md")]
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
