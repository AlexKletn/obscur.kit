//! Per-item grid alignment override (`justify-self` / `align-self`).

use crate::model::AlignItems;

/// How a single grid item aligns within its cell on one axis, overriding the
/// container's [`AlignItems`] default (spec §4.1 grid, `justify-self` /
/// `align-self`).
///
/// [`Auto`](Self::Auto) — the default — inherits the container's items default
/// (`justify-items` for the inline axis, `align-items` for the block axis); the
/// concrete values position the item within its cell exactly like the matching
/// [`AlignItems`] value.
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
