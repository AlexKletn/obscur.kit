//! `GridTemplate`: the explicit column and row track lists of a grid.

use super::TrackSize;

/// The explicit track template of a grid: its column and row sizing functions.
///
/// A `GridTemplate` is the CSS `grid-template-columns` / `grid-template-rows`
/// pair, each a list of [`TrackSize`]s (spec §4.1). Columns drive the horizontal
/// track sizing; rows drive the vertical.
///
/// # Implicit rows
///
/// `rows` may be **empty** (or shorter than the number of rows the items occupy).
/// Rows past the end of the explicit list are *implicit* and are sized as
/// [`TrackSize::Auto`] — as tall as the tallest item placed in them. An empty
/// `rows` list therefore yields an all-auto-row grid, which is the common
/// "columns only" template.
///
/// `columns` should be non-empty; the grid algorithm treats an empty column list
/// as a single flexible column so layout always has at least one column to place
/// items into.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GridTemplate {
    columns: Vec<TrackSize>,
    rows: Vec<TrackSize>,
}

impl GridTemplate {
    /// Builds a template from explicit `columns` and `rows` track lists.
    ///
    /// Pass an empty `rows` vector for a columns-only template whose rows are all
    /// implicit (auto).
    #[must_use]
    pub fn new(columns: Vec<TrackSize>, rows: Vec<TrackSize>) -> Self {
        Self { columns, rows }
    }

    /// Builds a columns-only template; every row is implicit (auto).
    #[must_use]
    pub fn columns_only(columns: Vec<TrackSize>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
        }
    }

    /// The explicit column tracks.
    #[must_use]
    pub fn columns(&self) -> &[TrackSize] {
        &self.columns
    }

    /// The explicit row tracks (may be empty → all rows implicit/auto).
    #[must_use]
    pub fn rows(&self) -> &[TrackSize] {
        &self.rows
    }
}

#[cfg(test)]
mod tests {
    use super::{GridTemplate, TrackSize};

    #[test]
    fn stores_columns_and_rows() {
        let t = GridTemplate::new(
            vec![TrackSize::Fixed(100.0), TrackSize::Fr(1.0)],
            vec![TrackSize::Auto],
        );
        assert_eq!(t.columns().len(), 2);
        assert_eq!(t.rows(), &[TrackSize::Auto]);
    }

    #[test]
    fn columns_only_has_empty_rows() {
        let t = GridTemplate::columns_only(vec![TrackSize::Fr(1.0)]);
        assert!(t.rows().is_empty());
        assert_eq!(t.columns(), &[TrackSize::Fr(1.0)]);
    }

    #[test]
    fn default_is_empty() {
        let t = GridTemplate::default();
        assert!(t.columns().is_empty());
        assert!(t.rows().is_empty());
    }
}
