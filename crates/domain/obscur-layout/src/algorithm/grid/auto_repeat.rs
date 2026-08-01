#![doc = include_str!("auto_repeat.doc/_module.md")]

use super::TrackSize;

#[doc = include_str!("auto_repeat.doc/AutoRepeatMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoRepeatMode {
    #[doc = include_str!("auto_repeat.doc/AutoFill.md")]
    AutoFill,
    #[doc = include_str!("auto_repeat.doc/AutoFit.md")]
    AutoFit,
}

#[doc = include_str!("auto_repeat.doc/AutoRepeat.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct AutoRepeat {
    mode: AutoRepeatMode,
    tracks: Vec<TrackSize>,
}

impl AutoRepeat {
    #[doc = include_str!("auto_repeat.doc/new.md")]
    #[must_use]
    pub fn new(mode: AutoRepeatMode, tracks: Vec<TrackSize>) -> Self {
        Self { mode, tracks }
    }

    #[doc = include_str!("auto_repeat.doc/mode.md")]
    #[must_use]
    pub const fn mode(&self) -> AutoRepeatMode {
        self.mode
    }

    #[doc = include_str!("auto_repeat.doc/tracks.md")]
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
