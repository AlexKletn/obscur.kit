#![doc = include_str!("grid_track_list.doc/_module.md")]

use crate::property::GridTrack;

#[doc = include_str!("grid_track_list.doc/GridTrackList.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct GridTrackList {
    #[doc = include_str!("grid_track_list.doc/tracks.md")]
    tracks: Box<[GridTrack]>,
    #[doc = include_str!("grid_track_list.doc/line_names.md")]
    line_names: Box<[Box<[Box<str>]>]>,
}

impl GridTrackList {
    #[doc = include_str!("grid_track_list.doc/new.md")]
    #[must_use]
    pub fn new(tracks: Box<[GridTrack]>, line_names: Box<[Box<[Box<str>]>]>) -> Self {
        Self { tracks, line_names }
    }

    #[doc = include_str!("grid_track_list.doc/from_tracks.md")]
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

    #[doc = include_str!("grid_track_list.doc/tracks_2.md")]
    #[must_use]
    pub fn tracks(&self) -> &[GridTrack] {
        &self.tracks
    }

    #[doc = include_str!("grid_track_list.doc/uses_ch.md")]
    #[must_use]
    pub fn uses_ch(&self) -> bool {
        self.tracks.iter().any(GridTrack::uses_ch)
    }

    #[doc = include_str!("grid_track_list.doc/line_names_2.md")]
    #[must_use]
    pub fn line_names(&self) -> &[Box<[Box<str>]>] {
        &self.line_names
    }

    #[doc = include_str!("grid_track_list.doc/into_tracks.md")]
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
