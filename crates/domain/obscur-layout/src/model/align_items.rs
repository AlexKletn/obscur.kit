#![doc = include_str!("align_items.doc/_module.md")]

#[doc = include_str!("align_items.doc/AlignItems.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlignItems {
    #[doc = include_str!("align_items.doc/FlexStart.md")]
    FlexStart,
    #[doc = include_str!("align_items.doc/FlexEnd.md")]
    FlexEnd,
    #[doc = include_str!("align_items.doc/Start.md")]
    Start,
    #[doc = include_str!("align_items.doc/End.md")]
    End,
    #[doc = include_str!("align_items.doc/Center.md")]
    Center,
    #[doc = include_str!("align_items.doc/Stretch.md")]
    #[default]
    Stretch,
}

#[cfg(test)]
mod tests {
    use super::AlignItems;

    #[test]
    fn default_is_stretch() {
        assert_eq!(AlignItems::default(), AlignItems::Stretch);
    }

    #[test]
    fn variants_are_distinct() {
        assert_ne!(AlignItems::FlexStart, AlignItems::FlexEnd);
        assert_ne!(AlignItems::Center, AlignItems::Stretch);
        assert_ne!(AlignItems::Start, AlignItems::End);
        assert_ne!(AlignItems::Start, AlignItems::FlexStart);
    }
}
