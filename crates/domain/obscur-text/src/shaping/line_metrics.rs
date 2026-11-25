//! A font's vertical line metrics at a concrete size (spec §5.10).

#[doc = include_str!("../../docs/api/shaping/line_metrics.doc/LineMetrics.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineMetrics {
    /// The ascent above the baseline, in logical pixels (non-negative).
    ascent: f32,
    /// The descent below the baseline, in logical pixels, as a positive
    /// magnitude.
    descent: f32,
    /// Extra leading between lines, in logical pixels.
    line_gap: f32,
}

impl LineMetrics {
    /// The fraction of the font size used as the ascent by the idealized default
    /// ([`idealized`](Self::idealized)).
    pub const IDEAL_ASCENT_RATIO: f32 = 0.8;
    /// The fraction of the font size used as the descent by the idealized default
    /// ([`idealized`](Self::idealized)).
    pub const IDEAL_DESCENT_RATIO: f32 = 0.2;

    /// Creates line metrics from an `ascent`, a positive-magnitude `descent`, and
    /// a `line_gap`, all in logical pixels.
    #[must_use]
    pub const fn new(ascent: f32, descent: f32, line_gap: f32) -> Self {
        Self {
            ascent,
            descent,
            line_gap,
        }
    }

    #[doc = include_str!("../../docs/api/shaping/line_metrics.doc/idealized.md")]
    #[must_use]
    pub fn idealized(font_size: f32) -> Self {
        Self::new(
            font_size * Self::IDEAL_ASCENT_RATIO,
            font_size * Self::IDEAL_DESCENT_RATIO,
            0.0,
        )
    }

    /// Returns the ascent above the baseline, in logical pixels.
    #[must_use]
    pub const fn ascent(&self) -> f32 {
        self.ascent
    }

    /// Returns the descent below the baseline, in logical pixels (positive
    /// magnitude).
    #[must_use]
    pub const fn descent(&self) -> f32 {
        self.descent
    }

    /// Returns the extra leading between lines, in logical pixels.
    #[must_use]
    pub const fn line_gap(&self) -> f32 {
        self.line_gap
    }

    /// Returns the natural line height: `ascent + descent + line_gap`, in logical
    /// pixels.
    #[must_use]
    pub fn line_height(&self) -> f32 {
        self.ascent + self.descent + self.line_gap
    }

    /// Returns the baseline offset from the top of the line box: the
    /// [`ascent`](Self::ascent), in logical pixels.
    #[must_use]
    pub const fn baseline(&self) -> f32 {
        self.ascent
    }
}

#[cfg(test)]
mod tests {
    use super::LineMetrics;

    #[test]
    fn accessors_return_constructed_values() {
        let metrics = LineMetrics::new(12.0, 3.0, 1.0);
        assert_eq!(metrics.ascent(), 12.0);
        assert_eq!(metrics.descent(), 3.0);
        assert_eq!(metrics.line_gap(), 1.0);
    }

    #[test]
    fn line_height_sums_all_three_bands() {
        let metrics = LineMetrics::new(12.0, 3.0, 1.0);
        assert_eq!(metrics.line_height(), 16.0);
    }

    #[test]
    fn baseline_is_the_ascent() {
        let metrics = LineMetrics::new(12.0, 3.0, 1.0);
        assert_eq!(metrics.baseline(), 12.0);
    }

    #[test]
    fn idealized_gives_size_as_line_height_and_point_eight_ascent() {
        let metrics = LineMetrics::idealized(20.0);
        assert_eq!(metrics.ascent(), 16.0); // 0.8 * 20
        assert_eq!(metrics.descent(), 4.0); // 0.2 * 20
        assert_eq!(metrics.line_gap(), 0.0);
        // line_height == font_size exactly.
        assert_eq!(metrics.line_height(), 20.0);
        assert_eq!(metrics.baseline(), 16.0);
    }
}
