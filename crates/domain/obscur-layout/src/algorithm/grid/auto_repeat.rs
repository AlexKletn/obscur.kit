//! `AutoRepeat`: a `repeat(auto-fill | auto-fit, <track-list>)` pattern.

use super::TrackSize;

/// Which auto-repeat mode a [`AutoRepeat`] pattern uses (CSS §4.1).
///
/// Both modes compute the *same* repetition count from the container content
/// extent (as many whole copies of the pattern as fit); they differ only in what
/// happens to a generated track that ends up with no placed item:
///
/// * [`AutoFill`](AutoRepeatMode::AutoFill) — empty tracks are **kept**; they
///   occupy their sizing-function's space just like any other track.
/// * [`AutoFit`](AutoRepeatMode::AutoFit) — empty tracks **collapse** to `0` after
///   placement and the gaps around them collapse too, so the occupied tracks pack
///   together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoRepeatMode {
    /// `repeat(auto-fill, …)`: keep empty repeated tracks (they occupy space).
    AutoFill,
    /// `repeat(auto-fit, …)`: collapse empty repeated tracks to `0` after placement.
    AutoFit,
}

/// A `repeat(auto-fill | auto-fit, <track-list>)` auto-repetition (CSS §4.1).
///
/// Unlike an integer `repeat(N, …)` — which the CSS layer expands to `N` flat
/// copies up front — an auto-repeat's repetition count depends on the grid
/// container's content extent along the axis, which is only known inside the
/// layout algorithm. So the pattern is carried symbolically here and expanded by
/// [`TrackList::expand`](super::TrackList::expand) at track-sizing time: as many
/// whole copies of `tracks` as fit the content extent.
///
/// The pattern `tracks` are expected to be fixed- or percentage-sized (an `fr` or
/// `auto` track has no definite size to fit repetitions against); a non-definite
/// pattern track contributes `0` to the fitted extent (a documented fallback) so
/// the count stays finite and deterministic.
#[derive(Debug, Clone, PartialEq)]
pub struct AutoRepeat {
    mode: AutoRepeatMode,
    tracks: Vec<TrackSize>,
}

impl AutoRepeat {
    /// Builds an auto-repeat from its `mode` and its repeated `tracks` pattern.
    #[must_use]
    pub fn new(mode: AutoRepeatMode, tracks: Vec<TrackSize>) -> Self {
        Self { mode, tracks }
    }

    /// The auto-repeat mode (`auto-fill` keeps empty tracks; `auto-fit` collapses).
    #[must_use]
    pub const fn mode(&self) -> AutoRepeatMode {
        self.mode
    }

    /// The repeated track pattern (one whole copy).
    #[must_use]
    pub fn tracks(&self) -> &[TrackSize] {
        &self.tracks
    }
}

#[cfg(test)]
mod tests {
    use super::{AutoRepeat, AutoRepeatMode};
    use crate::algorithm::TrackSize;

    #[test]
    fn carries_mode_and_pattern() {
        let r = AutoRepeat::new(AutoRepeatMode::AutoFill, vec![TrackSize::Fixed(100.0)]);
        assert_eq!(r.mode(), AutoRepeatMode::AutoFill);
        assert_eq!(r.tracks(), &[TrackSize::Fixed(100.0)]);
    }

    #[test]
    fn modes_are_distinct() {
        assert_ne!(AutoRepeatMode::AutoFill, AutoRepeatMode::AutoFit);
    }
}
