#![doc = include_str!("grid_template.doc/_module.md")]

use super::{TrackList, TrackSize};

#[doc = include_str!("grid_template.doc/GridTemplate.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GridTemplate {
    columns: TrackList,
    rows: TrackList,
    auto_columns: Vec<TrackSize>,
    auto_rows: Vec<TrackSize>,
}

impl GridTemplate {
    #[doc = include_str!("grid_template.doc/new.md")]
    #[must_use]
    pub fn new(columns: Vec<TrackSize>, rows: Vec<TrackSize>) -> Self {
        Self {
            columns: TrackList::from_tracks(columns),
            rows: TrackList::from_tracks(rows),
            auto_columns: Vec::new(),
            auto_rows: Vec::new(),
        }
    }

    #[doc = include_str!("grid_template.doc/from_track_lists.md")]
    #[must_use]
    pub fn from_track_lists(columns: TrackList, rows: TrackList) -> Self {
        Self {
            columns,
            rows,
            auto_columns: Vec::new(),
            auto_rows: Vec::new(),
        }
    }

    #[doc = include_str!("grid_template.doc/columns_only.md")]
    #[must_use]
    pub fn columns_only(columns: Vec<TrackSize>) -> Self {
        Self {
            columns: TrackList::from_tracks(columns),
            rows: TrackList::default(),
            auto_columns: Vec::new(),
            auto_rows: Vec::new(),
        }
    }

    #[doc = include_str!("grid_template.doc/with_auto_tracks.md")]
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

    #[doc = include_str!("grid_template.doc/columns.md")]
    #[must_use]
    pub const fn columns(&self) -> &TrackList {
        &self.columns
    }

    #[doc = include_str!("grid_template.doc/rows.md")]
    #[must_use]
    pub const fn rows(&self) -> &TrackList {
        &self.rows
    }

    #[doc = include_str!("grid_template.doc/auto_columns.md")]
    #[must_use]
    pub fn auto_columns(&self) -> &[TrackSize] {
        &self.auto_columns
    }

    #[doc = include_str!("grid_template.doc/auto_rows.md")]
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
