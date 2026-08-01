#![doc = include_str!("position.doc/_module.md")]

#[doc = include_str!("position.doc/Position.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Position {
    #[doc = include_str!("position.doc/Static.md")]
    #[default]
    Static,
    #[doc = include_str!("position.doc/Relative.md")]
    Relative,
    #[doc = include_str!("position.doc/Absolute.md")]
    Absolute,
    #[doc = include_str!("position.doc/Fixed.md")]
    Fixed,
    #[doc = include_str!("position.doc/Sticky.md")]
    Sticky,
}

impl Position {
    #[doc = include_str!("position.doc/is_relative.md")]
    #[must_use]
    pub const fn is_relative(self) -> bool {
        matches!(self, Self::Relative)
    }

    #[doc = include_str!("position.doc/is_out_of_flow.md")]
    #[must_use]
    pub const fn is_out_of_flow(self) -> bool {
        matches!(self, Self::Absolute | Self::Fixed)
    }

    #[doc = include_str!("position.doc/is_sticky.md")]
    #[must_use]
    pub const fn is_sticky(self) -> bool {
        matches!(self, Self::Sticky)
    }

    #[doc = include_str!("position.doc/is_positioned.md")]
    #[must_use]
    pub const fn is_positioned(self) -> bool {
        !matches!(self, Self::Static)
    }
}

#[cfg(test)]
mod tests {
    use super::Position;

    #[test]
    fn default_is_static() {
        assert_eq!(Position::default(), Position::Static);
    }

    #[test]
    fn only_relative_offsets_in_flow() {
        assert!(Position::Relative.is_relative());
        assert!(!Position::Static.is_relative());
        // Absolute/Fixed are out of flow, not offset in flow.
        assert!(!Position::Absolute.is_relative());
        assert!(!Position::Fixed.is_relative());
        // Sticky is in flow, but is NOT a plain relative offset.
        assert!(!Position::Sticky.is_relative());
    }

    #[test]
    fn only_absolute_and_fixed_are_out_of_flow() {
        assert!(Position::Absolute.is_out_of_flow());
        assert!(Position::Fixed.is_out_of_flow());
        assert!(!Position::Static.is_out_of_flow());
        assert!(!Position::Relative.is_out_of_flow());
        // Sticky stays IN flow (siblings unaffected), like relative.
        assert!(!Position::Sticky.is_out_of_flow());
    }

    #[test]
    fn only_sticky_is_sticky() {
        assert!(Position::Sticky.is_sticky());
        assert!(!Position::Static.is_sticky());
        assert!(!Position::Relative.is_sticky());
        assert!(!Position::Absolute.is_sticky());
        assert!(!Position::Fixed.is_sticky());
    }

    #[test]
    fn every_scheme_but_static_is_positioned() {
        assert!(!Position::Static.is_positioned());
        assert!(Position::Relative.is_positioned());
        assert!(Position::Absolute.is_positioned());
        assert!(Position::Fixed.is_positioned());
        // Sticky establishes a containing block for its descendants, like relative.
        assert!(Position::Sticky.is_positioned());
    }
}
