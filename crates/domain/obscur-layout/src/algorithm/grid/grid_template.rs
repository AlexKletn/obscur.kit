//! `GridTemplate`: the column and row track lists of a grid.

use super::{TrackList, TrackSize};

/// The track template of a grid: its column and row sizing functions.
///
/// A `GridTemplate` is the CSS `grid-template-columns` / `grid-template-rows`
/// pair, each a [`TrackList`] (spec §4.1) — explicit tracks plus an optional
/// `repeat(auto-fill | auto-fit, …)` auto-repetition. Columns drive the horizontal
/// track sizing; rows drive the vertical. The auto-repeat (if any) is expanded to
/// a concrete track vector inside the layout algorithm, where the container's
/// content extent — the basis for the repetition count — is known.
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
///
/// # Implicit-track sizes (`grid-auto-columns` / `grid-auto-rows`)
///
/// [`auto_columns`](Self::auto_columns) / [`auto_rows`](Self::auto_rows) hold the
/// sizing functions for *implicit* tracks — those created past the explicit
/// template by placement / auto-flow. The list is **cycled**: the n-th implicit
/// track on an axis uses index `n % len`. An **empty** list means the default
/// single content-sized `auto` (byte-identical to the historical behaviour where
/// every implicit track was `Auto`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GridTemplate {
    columns: TrackList,
    rows: TrackList,
    auto_columns: Vec<TrackSize>,
    auto_rows: Vec<TrackSize>,
}

impl GridTemplate {
    /// Builds a template from explicit `columns` and `rows` track vectors (no
    /// auto-repeat).
    ///
    /// Pass an empty `rows` vector for a columns-only template whose rows are all
    /// implicit (auto). This is the common no-repeat constructor; each vector
    /// becomes a [`TrackList::from_tracks`].
    #[must_use]
    pub fn new(columns: Vec<TrackSize>, rows: Vec<TrackSize>) -> Self {
        Self {
            columns: TrackList::from_tracks(columns),
            rows: TrackList::from_tracks(rows),
            auto_columns: Vec::new(),
            auto_rows: Vec::new(),
        }
    }

    /// Builds a template from column and row [`TrackList`]s (each of which may
    /// carry an auto-repeat).
    #[must_use]
    pub fn from_track_lists(columns: TrackList, rows: TrackList) -> Self {
        Self {
            columns,
            rows,
            auto_columns: Vec::new(),
            auto_rows: Vec::new(),
        }
    }

    /// Builds a columns-only template; every row is implicit (auto).
    #[must_use]
    pub fn columns_only(columns: Vec<TrackSize>) -> Self {
        Self {
            columns: TrackList::from_tracks(columns),
            rows: TrackList::default(),
            auto_columns: Vec::new(),
            auto_rows: Vec::new(),
        }
    }

    /// Returns a copy with the implicit-track sizing lists (`grid-auto-columns` /
    /// `grid-auto-rows`) replaced.
    ///
    /// Each list sizes the *implicit* tracks on its axis, cycled by the n-th
    /// implicit track using index `n % len`. An empty list keeps the default
    /// content-sized `auto` for that axis (byte-identical to the historical
    /// all-`Auto` implicit tracks).
    #[must_use]
    pub fn with_auto_tracks(
        mut self,
        auto_columns: Vec<TrackSize>,
        auto_rows: Vec<TrackSize>,
    ) -> Self {
        self.auto_columns = auto_columns;
        self.auto_rows = auto_rows;
        self
    }

    /// The column track list.
    #[must_use]
    pub const fn columns(&self) -> &TrackList {
        &self.columns
    }

    /// The row track list (may be empty → all rows implicit/auto).
    #[must_use]
    pub const fn rows(&self) -> &TrackList {
        &self.rows
    }

    /// The `grid-auto-columns` sizes for implicit column tracks (empty → `auto`).
    #[must_use]
    pub fn auto_columns(&self) -> &[TrackSize] {
        &self.auto_columns
    }

    /// The `grid-auto-rows` sizes for implicit row tracks (empty → `auto`).
    #[must_use]
    pub fn auto_rows(&self) -> &[TrackSize] {
        &self.auto_rows
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
        assert_eq!(t.columns().leading().len(), 2);
        assert_eq!(t.rows().leading(), &[TrackSize::Auto]);
    }

    #[test]
    fn columns_only_has_empty_rows() {
        let t = GridTemplate::columns_only(vec![TrackSize::Fr(1.0)]);
        assert!(t.rows().is_empty());
        assert_eq!(t.columns().leading(), &[TrackSize::Fr(1.0)]);
    }

    #[test]
    fn default_is_empty() {
        let t = GridTemplate::default();
        assert!(t.columns().is_empty());
        assert!(t.rows().is_empty());
        // Default implicit-track sizing is empty → the algorithm treats it as `auto`.
        assert!(t.auto_columns().is_empty());
        assert!(t.auto_rows().is_empty());
    }

    #[test]
    fn with_auto_tracks_stores_implicit_sizes() {
        let t = GridTemplate::columns_only(vec![TrackSize::Fr(1.0)])
            .with_auto_tracks(vec![TrackSize::Fixed(50.0)], vec![TrackSize::Fixed(40.0)]);
        assert_eq!(t.auto_columns(), &[TrackSize::Fixed(50.0)]);
        assert_eq!(t.auto_rows(), &[TrackSize::Fixed(40.0)]);
    }
}
