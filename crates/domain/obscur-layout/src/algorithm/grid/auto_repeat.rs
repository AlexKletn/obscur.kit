//! `AutoRepeat`: a `repeat(auto-fill | auto-fit, <track-list>)` pattern.

use super::TrackSize;

use super::AutoRepeatMode;

#[doc = include_str!("../../../docs/api/algorithm/grid/auto_repeat.doc/AutoRepeat.md")]
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
