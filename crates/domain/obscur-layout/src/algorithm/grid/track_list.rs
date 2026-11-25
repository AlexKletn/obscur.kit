//! `TrackList`: one axis of a grid template — explicit tracks plus an optional
//! `repeat(auto-fill | auto-fit, …)` auto-repetition (CSS §4.1).

use super::{AutoRepeat, AutoRepeatMode, TrackSize};

/// The largest repetition count an auto-repeat may expand to, so a pathological
/// container extent / pattern size cannot blow the track list up unboundedly.
const MAX_AUTO_REPEAT: usize = 10_000;

#[doc = include_str!("../../../docs/api/algorithm/grid/track_list.doc/TrackList.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrackList {
    leading: Vec<TrackSize>,
    auto_repeat: Option<AutoRepeat>,
    trailing: Vec<TrackSize>,
}

impl TrackList {
    /// Builds a track list with an explicit `leading` run, an optional
    /// `auto_repeat`, and an explicit `trailing` run.
    #[must_use]
    pub fn new(
        leading: Vec<TrackSize>,
        auto_repeat: Option<AutoRepeat>,
        trailing: Vec<TrackSize>,
    ) -> Self {
        Self {
            leading,
            auto_repeat,
            trailing,
        }
    }

    #[doc = include_str!("../../../docs/api/algorithm/grid/track_list.doc/from_tracks.md")]
    #[must_use]
    pub fn from_tracks(tracks: Vec<TrackSize>) -> Self {
        Self {
            leading: tracks,
            auto_repeat: None,
            trailing: Vec::new(),
        }
    }

    /// The leading explicit tracks (before any auto-repeat).
    #[must_use]
    pub fn leading(&self) -> &[TrackSize] {
        &self.leading
    }

    /// The auto-repeat pattern, if this axis has one.
    #[must_use]
    pub const fn auto_repeat(&self) -> Option<&AutoRepeat> {
        self.auto_repeat.as_ref()
    }

    /// The trailing explicit tracks (after any auto-repeat).
    #[must_use]
    pub fn trailing(&self) -> &[TrackSize] {
        &self.trailing
    }

    /// Whether the list has no explicit tracks and no auto-repeat.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.leading.is_empty() && self.auto_repeat.is_none() && self.trailing.is_empty()
    }

    /// A representative explicit track count: the explicit tracks plus one copy of
    /// any auto-repeat pattern. Used only for the informational
    /// [`GridLayout::columns`](super::super::GridLayout::columns) reporting; the
    /// real per-content count comes from [`expand`](Self::expand).
    #[must_use]
    pub fn explicit_len(&self) -> usize {
        self.leading.len()
            + self.auto_repeat.as_ref().map_or(0, |r| r.tracks().len())
            + self.trailing.len()
    }

    #[doc = include_str!("../../../docs/api/algorithm/grid/track_list.doc/expand.md")]
    #[must_use]
    pub fn expand(
        &self,
        content_extent: f32,
        gap: f32,
    ) -> (Vec<TrackSize>, Option<(usize, usize)>) {
        self.expand_for_bounds(0.0, content_extent, gap)
    }

    /// Expands auto-repeat against a finite ceiling or, if absent, a floor.
    pub(crate) fn expand_for_bounds(
        &self,
        minimum: f32,
        maximum: f32,
        gap: f32,
    ) -> (Vec<TrackSize>, Option<(usize, usize)>) {
        let Some(repeat) = self.auto_repeat.as_ref() else {
            let mut out = Vec::with_capacity(self.leading.len() + self.trailing.len());
            out.extend(self.leading.iter().cloned());
            out.extend(self.trailing.iter().cloned());
            return (out, None);
        };
        let pattern = repeat.tracks();
        let count = repetition_count(
            &self.leading,
            pattern,
            &self.trailing,
            minimum,
            maximum,
            gap,
        );

        let mut out =
            Vec::with_capacity(self.leading.len() + pattern.len() * count + self.trailing.len());
        out.extend(self.leading.iter().cloned());
        let start = out.len();
        for _ in 0..count {
            out.extend(pattern.iter().cloned());
        }
        let end = out.len();
        out.extend(self.trailing.iter().cloned());

        let auto_fit = matches!(repeat.mode(), AutoRepeatMode::AutoFit).then_some((start, end));
        (out, auto_fit)
    }
}

/// Returns the definite extent used to count auto-repeat copies. A `minmax()`
/// uses its definite maximum floored by the minimum when both are definite.
/// When only one bound is definite, it supplies the extent.
fn definite_extent(track: &TrackSize, content_extent: f32) -> Option<f32> {
    match track {
        TrackSize::Fixed(px) => Some(px.max(0.0)),
        TrackSize::Percent(f) if content_extent.is_finite() => Some((f * content_extent).max(0.0)),
        TrackSize::MinMax(min, max) => {
            let minimum = definite_extent(min, content_extent);
            let maximum = definite_extent(max, content_extent);
            match (minimum, maximum) {
                (Some(min), Some(max)) => Some(max.max(min)),
                _ => maximum.or(minimum),
            }
        }
        _ => None,
    }
}

/// Finds the largest positive count that fits a finite ceiling beside the
/// surrounding tracks and gaps. If only a minimum is supplied, finds the smallest
/// positive count that reaches it. Repeated tracks have a one-pixel floor for
/// counting. An indefinite track extent yields one repetition.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn repetition_count(
    leading: &[TrackSize],
    pattern: &[TrackSize],
    trailing: &[TrackSize],
    minimum: f32,
    maximum: f32,
    gap: f32,
) -> usize {
    let fit_ceiling = maximum.is_finite();
    let content_extent = if fit_ceiling { maximum } else { minimum };
    if !content_extent.is_finite() || pattern.is_empty() {
        return 1;
    }
    let Some(pattern_size) = pattern
        .iter()
        .map(|t| definite_extent(t, content_extent).map(|size| size.max(1.0)))
        .sum::<Option<f32>>()
    else {
        return 1;
    };
    let Some(outer_size) = leading
        .iter()
        .chain(trailing)
        .map(|t| definite_extent(t, content_extent))
        .sum::<Option<f32>>()
    else {
        return 1;
    };
    let per = pattern_size + pattern.len() as f32 * gap;
    let outer = (leading.len() + trailing.len()) as f32;
    let free = content_extent + gap - outer_size - outer * gap;
    let copies = free / per;
    let n = if fit_ceiling {
        copies.floor()
    } else {
        copies.ceil()
    };
    (n as usize).clamp(1, MAX_AUTO_REPEAT)
}

#[cfg(test)]
mod tests {
    use super::TrackList;
    use crate::algorithm::TrackSize;
    use crate::algorithm::grid::{AutoRepeat, AutoRepeatMode};

    fn fill(pattern: Vec<TrackSize>) -> TrackList {
        TrackList::new(
            Vec::new(),
            Some(AutoRepeat::new(AutoRepeatMode::AutoFill, pattern)),
            Vec::new(),
        )
    }

    #[test]
    fn no_repeat_expands_to_flat_tracks() {
        let list = TrackList::from_tracks(vec![TrackSize::Fixed(100.0), TrackSize::Fr(1.0)]);
        let (tracks, auto_fit) = list.expand(500.0, 0.0);
        assert_eq!(tracks, vec![TrackSize::Fixed(100.0), TrackSize::Fr(1.0)]);
        assert!(auto_fit.is_none());
    }

    #[test]
    fn auto_fill_count_from_content_no_gap() {
        // 100px pattern on a 350px axis: 3 fit (3*100=300 <= 350; a 4th overflows).
        let (tracks, auto_fit) = fill(vec![TrackSize::Fixed(100.0)]).expand(350.0, 0.0);
        assert_eq!(tracks.len(), 3);
        assert!(
            auto_fit.is_none(),
            "auto-fill keeps empty tracks (no collapse range)"
        );
    }

    #[test]
    fn auto_fill_count_accounts_for_gap() {
        // floor((350 + 10) / (100 + 10)) = floor(360/110) = 3.
        let (tracks, _) = fill(vec![TrackSize::Fixed(100.0)]).expand(350.0, 10.0);
        assert_eq!(tracks.len(), 3);
    }

    #[test]
    fn auto_fill_count_accounts_for_leading_track() {
        // A leading 50px track leaves 300px: 3 more 100px tracks fit (50 + 300 = 350).
        let list = TrackList::new(
            vec![TrackSize::Fixed(50.0)],
            Some(AutoRepeat::new(
                AutoRepeatMode::AutoFill,
                vec![TrackSize::Fixed(100.0)],
            )),
            Vec::new(),
        );
        let (tracks, _) = list.expand(350.0, 0.0);
        assert_eq!(tracks.len(), 4); // 1 leading + 3 repeated
        assert_eq!(tracks[0], TrackSize::Fixed(50.0));
    }

    #[test]
    fn auto_fit_reports_collapse_range() {
        let list = TrackList::new(
            Vec::new(),
            Some(AutoRepeat::new(
                AutoRepeatMode::AutoFit,
                vec![TrackSize::Fixed(100.0)],
            )),
            Vec::new(),
        );
        let (tracks, auto_fit) = list.expand(350.0, 0.0);
        assert_eq!(tracks.len(), 3);
        assert_eq!(auto_fit, Some((0, 3)));
    }

    #[test]
    fn auto_fill_fits_the_definite_maximum_floored_by_the_minimum() {
        let minmax = |min, max| TrackSize::MinMax(Box::new(min), Box::new(max));
        let fixed = TrackSize::Fixed;
        for (track, expected) in [
            (minmax(fixed(40.0), fixed(100.0)), 3),
            (minmax(fixed(100.0), fixed(40.0)), 3),
            (minmax(TrackSize::Auto, fixed(100.0)), 3),
            (minmax(fixed(40.0), TrackSize::Fr(1.0)), 8),
        ] {
            assert_eq!(fill(vec![track]).expand(350.0, 0.0).0.len(), expected);
        }
    }

    #[test]
    fn a_zero_sized_repeat_uses_the_one_pixel_floor_for_counting() {
        let (tracks, _) = fill(vec![TrackSize::Fixed(0.0)]).expand(4.0, 0.0);
        assert_eq!(tracks, vec![TrackSize::Fixed(0.0); 4]);
    }

    #[test]
    fn non_definite_pattern_falls_back_to_single_repetition() {
        // An `fr` pattern has no definite size to fit; count clamps to 1.
        let (tracks, _) = fill(vec![TrackSize::Fr(1.0)]).expand(350.0, 10.0);
        assert_eq!(tracks.len(), 1);
    }
}
