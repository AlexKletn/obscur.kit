#![doc = include_str!("radial_size.doc/_module.md")]

use crate::property::Length;

#[doc = include_str!("radial_size.doc/RadialSize.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum RadialSize {
    #[doc = include_str!("radial_size.doc/ClosestSide.md")]
    ClosestSide,
    #[doc = include_str!("radial_size.doc/ClosestCorner.md")]
    ClosestCorner,
    #[doc = include_str!("radial_size.doc/FarthestSide.md")]
    FarthestSide,
    #[doc = include_str!("radial_size.doc/FarthestCorner.md")]
    #[default]
    FarthestCorner,
    #[doc = include_str!("radial_size.doc/Length.md")]
    Length(Length),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_farthest_corner() {
        assert_eq!(RadialSize::default(), RadialSize::FarthestCorner);
    }

    #[test]
    fn length_variant_carries_its_length() {
        assert_eq!(
            RadialSize::Length(Length::px(20.0)),
            RadialSize::Length(Length::px(20.0))
        );
    }
}
