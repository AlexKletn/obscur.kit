//! `GridTemplate`: the column and row track lists of a grid.

use obscur_identity::{FastMap, NodeId};

use super::{RepeatedGridLine, TrackList, TrackSize};

#[doc = include_str!("../../../docs/api/algorithm/grid/grid_template.doc/GridTemplate.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GridTemplate {
    columns: TrackList,
    rows: TrackList,
    auto_columns: Vec<TrackSize>,
    auto_rows: Vec<TrackSize>,
    area_columns: usize,
    area_rows: usize,
    repeated_lines: FastMap<NodeId, [Option<RepeatedGridLine>; 4]>,
}

impl GridTemplate {
    #[doc = include_str!("../../../docs/api/algorithm/grid/grid_template.doc/new.md")]
    #[must_use]
    pub fn new(columns: Vec<TrackSize>, rows: Vec<TrackSize>) -> Self {
        Self {
            columns: TrackList::from_tracks(columns),
            rows: TrackList::from_tracks(rows),
            ..Self::default()
        }
    }

    /// Builds a template from column and row [`TrackList`]s (each of which may
    /// carry an auto-repeat).
    #[must_use]
    pub fn from_track_lists(columns: TrackList, rows: TrackList) -> Self {
        Self {
            columns,
            rows,
            ..Self::default()
        }
    }

    /// Builds a columns-only template; every row is implicit (auto).
    #[must_use]
    pub fn columns_only(columns: Vec<TrackSize>) -> Self {
        Self {
            columns: TrackList::from_tracks(columns),
            rows: TrackList::default(),
            ..Self::default()
        }
    }

    #[doc = include_str!("../../../docs/api/algorithm/grid/grid_template.doc/with_auto_tracks.md")]
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

    /// Sets the explicit grid dimensions established by its named areas.
    #[must_use]
    pub const fn with_area_dimensions(mut self, columns: usize, rows: usize) -> Self {
        self.area_columns = columns;
        self.area_rows = rows;
        self
    }

    /// Returns the explicit column and row counts required by named areas.
    #[must_use]
    pub const fn area_dimensions(&self) -> (usize, usize) {
        (self.area_columns, self.area_rows)
    }

    /// Sets deferred column-start, column-end, row-start and row-end lookups.
    pub fn set_repeated_lines(&mut self, item: NodeId, lines: [Option<RepeatedGridLine>; 4]) {
        self.repeated_lines.insert(item, lines);
    }

    pub(in crate::algorithm) fn resolve_repeated_lines(
        &self,
        items: &[crate::algorithm::LayoutChild],
        expanded_columns: usize,
        expanded_rows: usize,
    ) -> Option<Vec<crate::algorithm::LayoutChild>> {
        if self.repeated_lines.is_empty() {
            return None;
        }
        let resolve = |line: &RepeatedGridLine, axis: &TrackList, expanded: usize| {
            let pattern = axis.auto_repeat()?.tracks().len();
            let expanded_repeat =
                expanded.checked_sub(axis.leading().len() + axis.trailing().len())?;
            line.resolve(axis.leading().len(), pattern, expanded_repeat)
        };
        let mut resolved = items.to_vec();
        for item in &mut resolved {
            let Some([col_start, col_end, row_start, row_end]) =
                self.repeated_lines.get(&item.node)
            else {
                continue;
            };
            let placement = &mut item.style.grid_placement;
            for (line, edge, axis, expanded) in [
                (
                    col_start,
                    &mut placement.column_start,
                    &self.columns,
                    expanded_columns,
                ),
                (
                    col_end,
                    &mut placement.column_end,
                    &self.columns,
                    expanded_columns,
                ),
                (
                    row_start,
                    &mut placement.row_start,
                    &self.rows,
                    expanded_rows,
                ),
                (row_end, &mut placement.row_end, &self.rows, expanded_rows),
            ] {
                if let Some(value) = line.as_ref().and_then(|line| resolve(line, axis, expanded)) {
                    *edge = value;
                }
            }
        }
        Some(resolved)
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
