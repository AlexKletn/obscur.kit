#![doc = include_str!("justify_content.doc/_module.md")]

#[doc = include_str!("justify_content.doc/JustifyContent.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum JustifyContent {
    #[doc = include_str!("justify_content.doc/FlexStart.md")]
    #[default]
    FlexStart,
    #[doc = include_str!("justify_content.doc/FlexEnd.md")]
    FlexEnd,
    #[doc = include_str!("justify_content.doc/Center.md")]
    Center,
    #[doc = include_str!("justify_content.doc/SpaceBetween.md")]
    SpaceBetween,
    #[doc = include_str!("justify_content.doc/SpaceAround.md")]
    SpaceAround,
    #[doc = include_str!("justify_content.doc/SpaceEvenly.md")]
    SpaceEvenly,
    #[doc = include_str!("justify_content.doc/Stretch.md")]
    Stretch,
}

#[cfg(test)]
mod tests {
    use super::JustifyContent;

    #[test]
    fn default_is_flex_start() {
        assert_eq!(JustifyContent::default(), JustifyContent::FlexStart);
    }

    #[test]
    fn variants_are_distinct() {
        assert_ne!(JustifyContent::SpaceBetween, JustifyContent::SpaceAround);
        assert_ne!(JustifyContent::Center, JustifyContent::SpaceEvenly);
    }
}
