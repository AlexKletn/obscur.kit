#![doc = include_str!("background_position.doc/_module.md")]

use crate::property::Length;

#[doc = include_str!("background_position.doc/BackgroundPosition.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackgroundPosition {
    #[doc = include_str!("background_position.doc/x.md")]
    x: Length,
    #[doc = include_str!("background_position.doc/y.md")]
    y: Length,
}

impl BackgroundPosition {
    #[doc = include_str!("background_position.doc/new.md")]
    #[must_use]
    pub const fn new(x: Length, y: Length) -> Self {
        Self { x, y }
    }

    #[doc = include_str!("background_position.doc/x_2.md")]
    #[must_use]
    pub const fn x(self) -> Length {
        self.x
    }

    #[doc = include_str!("background_position.doc/y_2.md")]
    #[must_use]
    pub const fn y(self) -> Length {
        self.y
    }
}

impl Default for BackgroundPosition {
    #[doc = include_str!("background_position.doc/default.md")]
    fn default() -> Self {
        Self {
            x: Length::Percent(0.0),
            y: Length::Percent(0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_top_left() {
        let pos = BackgroundPosition::default();
        assert_eq!(pos.x(), Length::Percent(0.0));
        assert_eq!(pos.y(), Length::Percent(0.0));
    }

    #[test]
    fn new_round_trips_axes() {
        let pos = BackgroundPosition::new(Length::Percent(100.0), Length::px(10.0));
        assert_eq!(pos.x(), Length::Percent(100.0));
        assert_eq!(pos.y(), Length::px(10.0));
    }
}
