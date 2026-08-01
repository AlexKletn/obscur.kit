#![doc = include_str!("track_list.doc/_module.md")]

use super::{AutoRepeat, AutoRepeatMode, TrackSize};

#[doc = include_str!("track_list.doc/MAX_AUTO_REPEAT.md")]
const MAX_AUTO_REPEAT: usize = 10_000;

#[doc = include_str!("track_list.doc/TrackList.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrackList {
    leading: Vec<TrackSize>,
    auto_repeat: Option<AutoRepeat>,
    trailing: Vec<TrackSize>,
}

impl TrackList {
    #[doc = include_str!("track_list.doc/new.md")]
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

    #[doc = include_str!("track_list.doc/from_tracks.md")]
    #[must_use]
    pub fn from_tracks(tracks: Vec<TrackSize>) -> Self {
        Self {
            leading: tracks,
            auto_repeat: None,
            trailing: Vec::new(),
        }
    }

    #[doc = include_str!("track_list.doc/leading.md")]
    #[must_use]
    pub fn leading(&self) -> &[TrackSize] {
        &self.leading
    }

    #[doc = include_str!("track_list.doc/auto_repeat.md")]
    #[must_use]
    pub const fn auto_repeat(&self) -> Option<&AutoRepeat> {
        self.auto_repeat.as_ref()
    }

    #[doc = include_str!("track_list.doc/trailing.md")]
    #[must_use]
    pub fn trailing(&self) -> &[TrackSize] {
        &self.trailing
    }

    #[doc = include_str!("track_list.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.leading.is_empty() && self.auto_repeat.is_none() && self.trailing.is_empty()
    }

    #[doc = include_str!("track_list.doc/explicit_len.md")]
    #[must_use]
    pub fn explicit_len(&self) -> usize {
        self.leading.len()
            + self.auto_repeat.as_ref().map_or(0, |r| r.tracks().len())
            + self.trailing.len()
    }

    #[doc = include_str!("track_list.doc/expand.md")]
    #[must_use]
    pub fn expand(
        &self,
        content_extent: f32,
        gap: f32,
    ) -> (Vec<TrackSize>, Option<(usize, usize)>) {
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

#[doc = include_str!("track_list.doc/definite_extent.md")]
fn definite_extent(track: &TrackSize, content_extent: f32) -> f32 {
    match track {
        TrackSize::Fixed(px) => px.max(0.0),
        TrackSize::Percent(f) => (f * content_extent).max(0.0),
        TrackSize::MinMax(min, _) => definite_extent(min, content_extent),
        TrackSize::Fr(_) | TrackSize::Auto | TrackSize::MinContent | TrackSize::MaxContent => 0.0,
    }
}

#[doc = include_str!("track_list.doc/repetition_count.md")]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn repetition_count(
    leading: &[TrackSize],
    pattern: &[TrackSize],
    trailing: &[TrackSize],
    content_extent: f32,
    gap: f32,
) -> usize {
    let definite_sum = |tracks: &[TrackSize]| -> f32 {
        tracks
            .iter()
            .map(|t| definite_extent(t, content_extent))
            .sum()
    };
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
    fn non_definite_pattern_falls_back_to_single_repetition() {
        // An `fr` pattern has no definite size to fit; count clamps to 1.
        let (tracks, _) = fill(vec![TrackSize::Fr(1.0)]).expand(350.0, 0.0);
        assert_eq!(tracks.len(), 1);
    }
}
