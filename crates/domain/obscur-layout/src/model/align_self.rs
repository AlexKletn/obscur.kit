#![doc = include_str!("align_self.doc/_module.md")]

use crate::model::AlignItems;

#[doc = include_str!("align_self.doc/AlignSelf.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlignSelf {
    #[doc = include_str!("align_self.doc/Auto.md")]
    #[default]
    Auto,
    #[doc = include_str!("align_self.doc/Start.md")]
    Start,
    #[doc = include_str!("align_self.doc/End.md")]
    End,
    #[doc = include_str!("align_self.doc/Center.md")]
    Center,
    #[doc = include_str!("align_self.doc/Stretch.md")]
    Stretch,
}

impl AlignSelf {
    #[doc = include_str!("align_self.doc/resolve.md")]
    #[must_use]
    pub const fn resolve(self, container: AlignItems) -> AlignItems {
        match self {
            Self::Auto => container,
            Self::Start => AlignItems::Start,
            Self::End => AlignItems::End,
            Self::Center => AlignItems::Center,
            Self::Stretch => AlignItems::Stretch,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AlignItems, AlignSelf};

    #[test]
    fn default_is_auto() {
        assert_eq!(AlignSelf::default(), AlignSelf::Auto);
    }

    #[test]
    fn auto_inherits_container_default() {
        assert_eq!(
            AlignSelf::Auto.resolve(AlignItems::Center),
            AlignItems::Center
        );
        assert_eq!(
            AlignSelf::Auto.resolve(AlignItems::Stretch),
            AlignItems::Stretch
        );
    }

    #[test]
    fn concrete_values_override_container() {
        assert_eq!(
            AlignSelf::Start.resolve(AlignItems::Stretch),
            AlignItems::Start
        );
        assert_eq!(AlignSelf::End.resolve(AlignItems::Center), AlignItems::End);
        assert_eq!(
            AlignSelf::Stretch.resolve(AlignItems::Start),
            AlignItems::Stretch
        );
    }
}
