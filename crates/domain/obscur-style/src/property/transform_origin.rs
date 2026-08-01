#![doc = include_str!("transform_origin.doc/_module.md")]

use crate::property::Length;

#[doc = include_str!("transform_origin.doc/TransformOrigin.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformOrigin {
    #[doc = include_str!("transform_origin.doc/x.md")]
    x: Length,
    #[doc = include_str!("transform_origin.doc/y.md")]
    y: Length,
}

impl TransformOrigin {
    #[doc = include_str!("transform_origin.doc/CENTER.md")]
    pub const CENTER: Self = Self {
        x: Length::Percent(50.0),
        y: Length::Percent(50.0),
    };

    #[doc = include_str!("transform_origin.doc/new.md")]
    #[must_use]
    pub const fn new(x: Length, y: Length) -> Self {
        Self { x, y }
    }

    #[doc = include_str!("transform_origin.doc/x_2.md")]
    #[must_use]
    pub const fn x(&self) -> Length {
        self.x
    }

    #[doc = include_str!("transform_origin.doc/y_2.md")]
    #[must_use]
    pub const fn y(&self) -> Length {
        self.y
    }
}

impl Default for TransformOrigin {
    #[doc = include_str!("transform_origin.doc/default.md")]
    fn default() -> Self {
        Self::CENTER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_is_fifty_percent() {
        assert_eq!(TransformOrigin::CENTER.x(), Length::Percent(50.0));
        assert_eq!(TransformOrigin::CENTER.y(), Length::Percent(50.0));
    }

    #[test]
    fn stores_components() {
        let o = TransformOrigin::new(Length::px(10.0), Length::Percent(100.0));
        assert_eq!(o.x(), Length::px(10.0));
        assert_eq!(o.y(), Length::Percent(100.0));
    }
}
