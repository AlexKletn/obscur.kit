//! `GridTrackList`: a style-local `grid-template-columns` / `-rows` value —
//! the ordered track sizing functions plus the names on each grid line.

use crate::property::GridTrack;

/// A grid container's `grid-template-columns` / `grid-template-rows` value: the
/// flat, author-order list of [`GridTrack`] sizing functions together with the
/// `[name]` line names that were interleaved with them (spec §4.1, §4.13).
///
/// A track list names lines as well as sizing tracks: `[sidebar-start] 200px
/// [sidebar-end main-start] 1fr [main-end]` sizes two tracks (`200px`, `1fr`) and
/// names the three grid lines around them. The sizes live in
/// [`tracks`](GridTrackList::tracks) and the names in
/// [`line_names`](GridTrackList::line_names), one bucket per grid line — so
/// `line_names.len() == tracks.len() + 1` whenever any name was authored (a track
/// list with no `[name]` group carries an **empty** `line_names`, keeping the
/// no-names path allocation-free and byte-identical to the pre-named-lines value).
///
/// A line can carry several names (`[a b]`), so each bucket is itself a list. A
/// child's `grid-column: <name>` is resolved against this list in the runtime, the
/// same way `grid-area: <name>` is resolved against the container's
/// `grid-template-areas`.
///
/// # Determinism
///
/// `GridTrackList` owns its data with no interior mutability, so a container's
/// template is a deterministic value with a stable [`PartialEq`]. Because
/// [`GridTrack`] carries `f32` data this derives [`PartialEq`] but not [`Eq`].
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
    /// Builds a track list from its tracks and per-line names.
    ///
    /// Pass an empty `line_names` for a list with no authored `[name]` groups.
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

    /// Returns `true` when any track in this list is sized with a `ch` term.
    ///
    /// Line names carry no lengths, so only the tracks are consulted. See
    /// [`GridTrack::uses_ch`].
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
