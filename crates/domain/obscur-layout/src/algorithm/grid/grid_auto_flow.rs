//! `GridAutoFlow`: a grid container's auto-placement flow mode (spec §4.5).

#[doc = include_str!("../../../docs/api/algorithm/grid/grid_auto_flow.doc/GridAutoFlow.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GridAutoFlow {
    /// `row` — row-major, sparse. The default.
    #[default]
    Row,
    /// `column` — column-major, sparse.
    Column,
    /// `row dense` — row-major, dense (backfills earlier holes).
    RowDense,
    /// `column dense` — column-major, dense (backfills earlier holes).
    ColumnDense,
}

impl GridAutoFlow {
    /// Whether items flow *column-major* (down a column before wrapping across).
    #[must_use]
    pub const fn is_column(self) -> bool {
        matches!(self, Self::Column | Self::ColumnDense)
    }

    /// Whether packing is *dense* (each auto item searched from the origin, so it
    /// may backfill an earlier hole).
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
