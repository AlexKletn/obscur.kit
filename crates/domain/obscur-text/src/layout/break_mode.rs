#![doc = include_str!("break_mode.doc/_module.md")]

#[doc = include_str!("break_mode.doc/OverflowWrap.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverflowWrap {
    #[doc = include_str!("break_mode.doc/Normal.md")]
    #[default]
    Normal,
    #[doc = include_str!("break_mode.doc/BreakWord.md")]
    BreakWord,
    #[doc = include_str!("break_mode.doc/Anywhere.md")]
    Anywhere,
}

impl OverflowWrap {
    #[doc = include_str!("break_mode.doc/allows_emergency_break.md")]
    #[must_use]
    pub const fn allows_emergency_break(self) -> bool {
        matches!(self, Self::BreakWord | Self::Anywhere)
    }
}

#[doc = include_str!("break_mode.doc/WordBreak.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WordBreak {
    #[doc = include_str!("break_mode.doc/Normal_2.md")]
    #[default]
    Normal,
    #[doc = include_str!("break_mode.doc/BreakAll.md")]
    BreakAll,
    #[doc = include_str!("break_mode.doc/KeepAll.md")]
    KeepAll,
}

impl WordBreak {
    #[doc = include_str!("break_mode.doc/breaks_all.md")]
    #[must_use]
    pub const fn breaks_all(self) -> bool {
        matches!(self, Self::BreakAll)
    }

    #[doc = include_str!("break_mode.doc/keeps_all.md")]
    #[must_use]
    pub const fn keeps_all(self) -> bool {
        matches!(self, Self::KeepAll)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_normal() {
        assert_eq!(OverflowWrap::default(), OverflowWrap::Normal);
        assert_eq!(WordBreak::default(), WordBreak::Normal);
    }

    #[test]
    fn overflow_wrap_emergency_predicate() {
        assert!(!OverflowWrap::Normal.allows_emergency_break());
        assert!(OverflowWrap::BreakWord.allows_emergency_break());
        assert!(OverflowWrap::Anywhere.allows_emergency_break());
    }

    #[test]
    fn word_break_predicates() {
        assert!(WordBreak::BreakAll.breaks_all());
        assert!(!WordBreak::Normal.breaks_all());
        assert!(WordBreak::KeepAll.keeps_all());
        assert!(!WordBreak::Normal.keeps_all());
    }
}
