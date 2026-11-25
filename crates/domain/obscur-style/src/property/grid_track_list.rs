//! `GridTrackList`: a style-local `grid-template-columns` / `-rows` value —
//! the ordered track sizing functions plus the names on each grid line.

use crate::property::GridTrack;

#[doc = include_str!("../../docs/api/property/grid_track_list.doc/GridTrackList.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct GridTrackList {
    /// The track sizing functions, in author order (already flat: any `repeat(…)`
    /// was expanded before this value was built).
    tracks: Box<[GridTrack]>,
    /// The names on each grid line, indexed by 0-based line position (line `i` is
    /// the `i`-th boundary, so line `0` is before the first track). Empty when no
    /// `[name]` group was authored; otherwise `tracks.len() + 1` buckets, each a
    /// (possibly empty) list of the names on that line.
    line_names: Box<[Box<[Box<str>]>]>,
}

impl GridTrackList {
    #[doc = include_str!("../../docs/api/property/grid_track_list.doc/new.md")]
    #[must_use]
    pub fn new(tracks: Box<[GridTrack]>, line_names: Box<[Box<[Box<str>]>]>) -> Self {
        Self { tracks, line_names }
    }

    /// Builds a track list with no line names from an ordered iterator of tracks.
    #[must_use]
    pub fn from_tracks<I>(tracks: I) -> Self
    where
        I: IntoIterator<Item = GridTrack>,
    {
        Self {
            tracks: tracks.into_iter().collect(),
            line_names: Box::from([]),
        }
    }

    /// The track sizing functions, in author order.
    #[must_use]
    pub fn tracks(&self) -> &[GridTrack] {
        &self.tracks
    }

    #[doc = include_str!("../../docs/api/property/grid_track_list.doc/uses_ch.md")]
    #[must_use]
    pub fn uses_ch(&self) -> bool {
        self.tracks.iter().any(GridTrack::uses_ch)
    }

    /// The names on each grid line, indexed by 0-based line position. Empty when
    /// no `[name]` group was authored.
    #[must_use]
    pub fn line_names(&self) -> &[Box<[Box<str>]>] {
        &self.line_names
    }

    /// Consumes the list, returning its tracks (dropping any line names).
    #[must_use]
    pub fn into_tracks(self) -> Box<[GridTrack]> {
        self.tracks
    }
}

#[cfg(test)]
mod tests {
    use super::GridTrackList;
    use crate::property::{GridTrack, Length};

    #[test]
    fn from_tracks_has_no_line_names() {
        let list = GridTrackList::from_tracks([GridTrack::Fr(1.0), GridTrack::Auto]);
        assert_eq!(list.tracks(), &[GridTrack::Fr(1.0), GridTrack::Auto]);
        assert!(list.line_names().is_empty());
    }

    #[test]
    fn new_carries_tracks_and_line_names() {
        let list = GridTrackList::new(
            Box::from([GridTrack::Fixed(Length::px(100.0)), GridTrack::Fr(1.0)]),
            Box::from([
                Box::from([Box::<str>::from("a")]) as Box<[Box<str>]>,
                Box::from([Box::<str>::from("b")]),
                Box::from([Box::<str>::from("c")]),
            ]),
        );
        assert_eq!(list.tracks().len(), 2);
        assert_eq!(list.line_names().len(), 3);
        assert_eq!(&*list.line_names()[0][0], "a");
        assert_eq!(&*list.line_names()[2][0], "c");
    }

    #[test]
    fn into_tracks_drops_names() {
        let list = GridTrackList::from_tracks([GridTrack::Auto]);
        assert_eq!(list.into_tracks().into_vec(), vec![GridTrack::Auto]);
    }
}
