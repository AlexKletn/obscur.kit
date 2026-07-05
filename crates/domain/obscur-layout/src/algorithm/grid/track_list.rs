//! `TrackList`: one axis of a grid template — explicit tracks plus an optional
//! `repeat(auto-fill | auto-fit, …)` auto-repetition (CSS §4.1).

use super::{AutoRepeat, AutoRepeatMode, TrackSize};

/// The largest repetition count an auto-repeat may expand to, so a pathological
/// container extent / pattern size cannot blow the track list up unboundedly.
const MAX_AUTO_REPEAT: usize = 10_000;

/// One axis of a grid template: leading explicit tracks, an optional auto-repeat,
/// and trailing explicit tracks (CSS §4.1 `grid-template-columns` / `-rows`).
///
/// A plain track list (no `repeat(auto-fill|auto-fit, …)`) is modelled with all
/// tracks in [`leading`](Self::leading) and no [`auto_repeat`](Self::auto_repeat)
/// — [`TrackList::from_tracks`] builds exactly that, so a template without an
/// auto-repeat expands **byte-identically** to its explicit track vector.
///
/// When an [`AutoRepeat`] is present the list reads
/// `leading ++ (N × pattern) ++ trailing`, where the repetition count `N` is
/// computed from the container's content extent along the axis at track-sizing
/// time by [`expand`](Self::expand). Only one auto-repeat is representable per
/// axis (as in CSS).
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

    /// Builds a track list from a flat explicit track vector (no auto-repeat).
    ///
    /// This is the common no-repeat case; the tracks become the `leading` run and
    /// the list expands to exactly this vector.
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

    /// Expands this list into a concrete track vector for a `content_extent`-wide
    /// axis with `gap` between tracks.
    ///
    /// With no auto-repeat this is simply `leading ++ trailing` and the second
    /// element is [`None`]. With an auto-repeat the pattern is repeated `N` times
    /// (§4.1): `N = max(1, floor((free + gap) / (pattern_extent + gap)))`, where
    /// `free` is the content extent minus the definite (fixed/percentage) size of
    /// the leading and trailing tracks and their gaps, and `pattern_extent` is the
    /// sum of the pattern's definite track sizes plus its interior gaps. The result
    /// is `leading ++ (N × pattern) ++ trailing`.
    ///
    /// The second element is `Some((start, end))` — the half-open index range of
    /// the generated pattern tracks — **only** for an `auto-fit` repeat, so the
    /// caller can collapse the empty ones after placement; it is [`None`] for
    /// `auto-fill` (whose empty tracks are kept).
    #[must_use]
    pub fn expand(&self, content_extent: f32, gap: f32) -> (Vec<TrackSize>, Option<(usize, usize)>) {
        let Some(repeat) = self.auto_repeat.as_ref() else {
            let mut out = Vec::with_capacity(self.leading.len() + self.trailing.len());
            out.extend(self.leading.iter().cloned());
            out.extend(self.trailing.iter().cloned());
            return (out, None);
        };
        let pattern = repeat.tracks();
        let count = repetition_count(&self.leading, pattern, &self.trailing, content_extent, gap);

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

/// The definite (fixed/percentage) extent a single track contributes when fitting
/// auto-repeat copies. An `fr` / `auto` track has no definite size — it cannot
/// bound a repetition count — so it contributes `0` (a documented fallback); a
/// `minmax` uses its `min` bound as the definite floor.
fn definite_extent(track: &TrackSize, content_extent: f32) -> f32 {
    match track {
        TrackSize::Fixed(px) => px.max(0.0),
        TrackSize::Percent(f) => (f * content_extent).max(0.0),
        TrackSize::MinMax(min, _) => definite_extent(min, content_extent),
        TrackSize::Fr(_) | TrackSize::Auto => 0.0,
    }
}

/// Computes the auto-repeat repetition count `N ≥ 1` that fits the pattern into
/// `content_extent` alongside the leading/trailing explicit tracks (§4.1).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn repetition_count(
    leading: &[TrackSize],
    pattern: &[TrackSize],
    trailing: &[TrackSize],
    content_extent: f32,
    gap: f32,
) -> usize {
    let definite_sum =
        |tracks: &[TrackSize]| -> f32 { tracks.iter().map(|t| definite_extent(t, content_extent)).sum() };
    let pattern_size = definite_sum(pattern);
    // `pattern_extent + gap`: one whole pattern plus the gap that precedes each copy.
    let per = pattern_size + pattern.len() as f32 * gap;
    if per <= 0.0 {
        return 1;
    }
    let outer = (leading.len() + trailing.len()) as f32;
    let free = content_extent + gap - definite_sum(leading) - definite_sum(trailing) - outer * gap;
    if free <= 0.0 {
        return 1;
    }
    let n = (free / per).floor();
    if !n.is_finite() || n < 1.0 {
        return 1;
    }
    (n as usize).clamp(1, MAX_AUTO_REPEAT)
}

#[cfg(test)]
mod tests {
    use super::TrackList;
    use crate::algorithm::grid::{AutoRepeat, AutoRepeatMode};
    use crate::algorithm::TrackSize;

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
        assert!(auto_fit.is_none(), "auto-fill keeps empty tracks (no collapse range)");
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
            Some(AutoRepeat::new(AutoRepeatMode::AutoFill, vec![TrackSize::Fixed(100.0)])),
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
            Some(AutoRepeat::new(AutoRepeatMode::AutoFit, vec![TrackSize::Fixed(100.0)])),
            Vec::new(),
        );
        let (tracks, auto_fit) = list.expand(350.0, 0.0);
        assert_eq!(tracks.len(), 3);
        assert_eq!(auto_fit, Some((0, 3)));
    }

    #[test]
    fn non_definite_pattern_falls_back_to_single_repetition() {
        // An `fr` pattern has no definite size to fit; count clamps to 1.
        let (tracks, _) = fill(vec![TrackSize::Fr(1.0)]).expand(350.0, 0.0);
        assert_eq!(tracks.len(), 1);
    }
}
