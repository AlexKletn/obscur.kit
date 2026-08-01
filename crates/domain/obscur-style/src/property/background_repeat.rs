#![doc = include_str!("background_repeat.doc/_module.md")]

#[doc = include_str!("background_repeat.doc/BackgroundRepeat.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackgroundRepeat {
    #[doc = include_str!("background_repeat.doc/Repeat.md")]
    #[default]
    Repeat,
    #[doc = include_str!("background_repeat.doc/NoRepeat.md")]
    NoRepeat,
    #[doc = include_str!("background_repeat.doc/RepeatX.md")]
    RepeatX,
    #[doc = include_str!("background_repeat.doc/RepeatY.md")]
    RepeatY,
}

impl BackgroundRepeat {
    #[doc = include_str!("background_repeat.doc/repeat_x.md")]
    #[must_use]
    pub const fn repeat_x(self) -> bool {
        matches!(self, Self::Repeat | Self::RepeatX)
    }

    #[doc = include_str!("background_repeat.doc/repeat_y.md")]
    #[must_use]
    pub const fn repeat_y(self) -> bool {
        matches!(self, Self::Repeat | Self::RepeatY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_repeats_both_axes() {
        let repeat = BackgroundRepeat::default();
        assert_eq!(repeat, BackgroundRepeat::Repeat);
        assert!(repeat.repeat_x());
        assert!(repeat.repeat_y());
    }

    #[test]
    fn axis_flags_match_variant() {
        assert!(!BackgroundRepeat::NoRepeat.repeat_x());
        assert!(!BackgroundRepeat::NoRepeat.repeat_y());
        assert!(BackgroundRepeat::RepeatX.repeat_x());
        assert!(!BackgroundRepeat::RepeatX.repeat_y());
        assert!(!BackgroundRepeat::RepeatY.repeat_x());
        assert!(BackgroundRepeat::RepeatY.repeat_y());
    }
}
