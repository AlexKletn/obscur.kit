//! How a line's glyphs are positioned horizontally within the paragraph width
//! (spec §5.10).

use crate::shaping::TextDirection;

/// The horizontal alignment applied to each laid-out line within the available
/// paragraph width (spec §5.10).
///
/// [`Start`](Self::Start) and [`End`](Self::End) are *direction-relative*: they
/// resolve to the leading or trailing edge according to the run's
/// [`TextDirection`], so `Start` hugs the left edge in left-to-right text and the
/// right edge in right-to-left text. [`Left`](Self::Left) and
/// [`Right`](Self::Right) are *absolute* and ignore direction.
/// [`Center`](Self::Center) centres the line, and [`Justify`](Self::Justify)
/// stretches inter-glyph spacing to fill the line (so its start offset is `0`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextAlignment {
    /// Align to the leading edge: left under left-to-right, right under
    /// right-to-left. The default.
    #[default]
    Start,
    /// Align to the trailing edge: right under left-to-right, left under
    /// right-to-left.
    End,
    /// Align to the absolute left edge, regardless of direction.
    Left,
    /// Align to the absolute right edge, regardless of direction.
    Right,
    /// Centre the line within the available width.
    Center,
    /// Stretch inter-glyph spacing to fill the available width; the line's start
    /// offset is `0`.
    Justify,
}

impl TextAlignment {
    /// Computes the horizontal start offset (in logical pixels) for a line of
    /// width `line_width` within `available_width`, under base `direction`
    /// (spec §5.10).
    ///
    /// The result is the x position of the line's leading glyph relative to the
    /// paragraph's left edge:
    ///
    /// - [`Left`](Self::Left) and [`Justify`](Self::Justify) return `0`.
    /// - [`Right`](Self::Right) returns `available_width - line_width`.
    /// - [`Center`](Self::Center) returns half of the remaining space.
    /// - [`Start`](Self::Start) returns `0` under left-to-right and the
    ///   right-aligned offset under right-to-left; [`End`](Self::End) is the
    ///   mirror of `Start`.
    ///
    /// When the line is wider than the available width the raw offset would be
    /// negative; it is clamped up to `0` so the line never starts left of the
    /// paragraph edge.
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
