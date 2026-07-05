//! `GridAutoFlow`: a grid container's auto-placement flow mode (spec §4.5).

/// How a grid container auto-places its fully-auto items (`grid-auto-flow`).
///
/// The flow mode has two independent facets, folded here into a small `Copy`
/// enum:
///
/// * **axis** — [`Row`](Self::Row)/[`RowDense`](Self::RowDense) fill *row-major*
///   (across each row, then wrap to the next row), while
///   [`Column`](Self::Column)/[`ColumnDense`](Self::ColumnDense) fill
///   *column-major* (down each column, then wrap to the next column).
/// * **packing** — the plain [`Row`]/[`Column`] modes pack *sparsely*: an
///   auto-placement cursor only ever moves forward, so a hole left behind (by a
///   definite item or a span that wrapped) is never backfilled. The
///   `*Dense` modes pack *densely*: every auto item is searched from the grid
///   origin, so a smaller later item backfills an earlier hole.
///
/// [`Default`] is [`Row`](Self::Row) — the CSS initial value and the engine's
/// historical row-major, non-backfilling behaviour — so a container that does
/// not author `grid-auto-flow` auto-places exactly as before.
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
