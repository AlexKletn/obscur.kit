#![doc = include_str!("selection_mode.doc/_module.md")]

#[doc = include_str!("selection_mode.doc/SelectionMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelectionMode {
    #[doc = include_str!("selection_mode.doc/None.md")]
    None,
    #[doc = include_str!("selection_mode.doc/Single.md")]
    Single,
    #[doc = include_str!("selection_mode.doc/Multiple.md")]
    Multiple,
}

impl SelectionMode {
    #[doc = include_str!("selection_mode.doc/allows_multiple.md")]
    #[must_use]
    pub const fn allows_multiple(&self) -> bool {
        matches!(self, Self::Multiple)
    }

    #[doc = include_str!("selection_mode.doc/is_enabled.md")]
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        matches!(self, Self::Single | Self::Multiple)
    }
}

impl Default for SelectionMode {
    #[doc = include_str!("selection_mode.doc/default.md")]
    fn default() -> Self {
        Self::Single
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_single() {
        assert_eq!(SelectionMode::default(), SelectionMode::Single);
    }

    #[test]
    fn allows_multiple_only_for_multiple() {
        assert!(SelectionMode::Multiple.allows_multiple());
        assert!(!SelectionMode::Single.allows_multiple());
        assert!(!SelectionMode::None.allows_multiple());
    }

    #[test]
    fn is_enabled_for_single_and_multiple() {
        assert!(SelectionMode::Single.is_enabled());
        assert!(SelectionMode::Multiple.is_enabled());
        assert!(!SelectionMode::None.is_enabled());
    }
}
