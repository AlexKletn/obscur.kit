//! Per-item grid alignment override (`justify-self` / `align-self`).

use crate::model::AlignItems;

#[doc = include_str!("../../docs/api/model/align_self.doc/AlignSelf.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlignSelf {
    /// Inherit the container's items default for this axis (the initial value).
    #[default]
    Auto,
    /// Align the item toward the start of the cell.
    Start,
    /// Align the item toward the end of the cell.
    End,
    /// Center the item within the cell.
    Center,
    /// Aligns the first baseline with the other items in its baseline-sharing group.
    Baseline,
    /// Aligns the last baseline with its baseline-sharing group.
    LastBaseline,
    /// Stretch the item to fill the cell.
    Stretch,
}

impl AlignSelf {
    /// Resolves the effective per-axis alignment against the container's items
    /// default: [`Auto`](Self::Auto) yields the container's value, every other
    /// value maps to the matching [`AlignItems`] variant.
    #[must_use]
    pub const fn resolve(self, container: AlignItems) -> AlignItems {
        match self {
            Self::Auto => container,
            Self::Start => AlignItems::Start,
            Self::End => AlignItems::End,
            Self::Center => AlignItems::Center,
            Self::Baseline => AlignItems::Baseline,
            Self::LastBaseline => AlignItems::LastBaseline,
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
