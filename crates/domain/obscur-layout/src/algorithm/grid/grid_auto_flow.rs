#![doc = include_str!("grid_auto_flow.doc/_module.md")]

#[doc = include_str!("grid_auto_flow.doc/GridAutoFlow.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GridAutoFlow {
    #[doc = include_str!("grid_auto_flow.doc/Row.md")]
    #[default]
    Row,
    #[doc = include_str!("grid_auto_flow.doc/Column.md")]
    Column,
    #[doc = include_str!("grid_auto_flow.doc/RowDense.md")]
    RowDense,
    #[doc = include_str!("grid_auto_flow.doc/ColumnDense.md")]
    ColumnDense,
}

impl GridAutoFlow {
    #[doc = include_str!("grid_auto_flow.doc/is_column.md")]
    #[must_use]
    pub const fn is_column(self) -> bool {
        matches!(self, Self::Column | Self::ColumnDense)
    }

    #[doc = include_str!("grid_auto_flow.doc/is_dense.md")]
    #[must_use]
    pub const fn is_dense(self) -> bool {
        matches!(self, Self::RowDense | Self::ColumnDense)
    }
}

#[cfg(test)]
mod tests {
    use super::GridAutoFlow;

    #[test]
    fn default_is_row_sparse() {
        let f = GridAutoFlow::default();
        assert_eq!(f, GridAutoFlow::Row);
        assert!(!f.is_column());
        assert!(!f.is_dense());
    }

    #[test]
    fn axis_and_packing_facets() {
        assert!(!GridAutoFlow::Row.is_column());
        assert!(GridAutoFlow::Column.is_column());
        assert!(GridAutoFlow::ColumnDense.is_column());
        assert!(!GridAutoFlow::RowDense.is_column());

        assert!(!GridAutoFlow::Row.is_dense());
        assert!(!GridAutoFlow::Column.is_dense());
        assert!(GridAutoFlow::RowDense.is_dense());
        assert!(GridAutoFlow::ColumnDense.is_dense());
    }
}
