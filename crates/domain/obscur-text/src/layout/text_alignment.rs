#![doc = include_str!("text_alignment.doc/_module.md")]

use crate::shaping::TextDirection;

#[doc = include_str!("text_alignment.doc/TextAlignment.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextAlignment {
    #[doc = include_str!("text_alignment.doc/Start.md")]
    #[default]
    Start,
    #[doc = include_str!("text_alignment.doc/End.md")]
    End,
    #[doc = include_str!("text_alignment.doc/Left.md")]
    Left,
    #[doc = include_str!("text_alignment.doc/Right.md")]
    Right,
    #[doc = include_str!("text_alignment.doc/Center.md")]
    Center,
    #[doc = include_str!("text_alignment.doc/Justify.md")]
    Justify,
}

impl TextAlignment {
    #[doc = include_str!("text_alignment.doc/x_offset.md")]
    #[must_use]
    pub const fn x_offset(
        self,
        line_width: f32,
        available_width: f32,
        direction: TextDirection,
    ) -> f32 {
        let remaining = available_width - line_width;
        let offset = match self {
            Self::Left | Self::Justify => 0.0,
            Self::Right => remaining,
            Self::Center => remaining / 2.0,
            Self::Start => {
                if direction.is_rtl() {
                    remaining
                } else {
                    0.0
                }
            }
            Self::End => {
                if direction.is_rtl() {
                    0.0
                } else {
                    remaining
                }
            }
        };
        // Clamp negatives to zero (line wider than the available width).
        if offset < 0.0 { 0.0 } else { offset }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LTR: TextDirection = TextDirection::LeftToRight;
    const RTL: TextDirection = TextDirection::RightToLeft;

    #[test]
    fn default_is_start() {
        assert_eq!(TextAlignment::default(), TextAlignment::Start);
    }

    #[test]
    fn left_and_justify_are_zero() {
        assert_eq!(TextAlignment::Left.x_offset(30.0, 100.0, LTR), 0.0);
        assert_eq!(TextAlignment::Justify.x_offset(30.0, 100.0, LTR), 0.0);
    }

    #[test]
    fn right_and_center() {
        assert_eq!(TextAlignment::Right.x_offset(30.0, 100.0, LTR), 70.0);
        assert_eq!(TextAlignment::Center.x_offset(30.0, 100.0, LTR), 35.0);
    }

    #[test]
    fn start_and_end_resolve_with_direction() {
        // LTR: Start hugs the left (0), End hugs the right (remaining).
        assert_eq!(TextAlignment::Start.x_offset(30.0, 100.0, LTR), 0.0);
        assert_eq!(TextAlignment::End.x_offset(30.0, 100.0, LTR), 70.0);
        // RTL: Start hugs the right (remaining), End hugs the left (0).
        assert_eq!(TextAlignment::Start.x_offset(30.0, 100.0, RTL), 70.0);
        assert_eq!(TextAlignment::End.x_offset(30.0, 100.0, RTL), 0.0);
    }

    #[test]
    fn negative_offset_is_clamped_to_zero() {
        // Line wider than available: every alignment clamps to 0.
        assert_eq!(TextAlignment::Right.x_offset(120.0, 100.0, LTR), 0.0);
        assert_eq!(TextAlignment::Center.x_offset(120.0, 100.0, LTR), 0.0);
        assert_eq!(TextAlignment::Start.x_offset(120.0, 100.0, RTL), 0.0);
    }
}
