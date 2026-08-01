#![doc = include_str!("line_metrics.doc/_module.md")]

#[doc = include_str!("line_metrics.doc/LineMetrics.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineMetrics {
    #[doc = include_str!("line_metrics.doc/ascent.md")]
    ascent: f32,
    #[doc = include_str!("line_metrics.doc/descent.md")]
    descent: f32,
    #[doc = include_str!("line_metrics.doc/line_gap.md")]
    line_gap: f32,
}

impl LineMetrics {
    #[doc = include_str!("line_metrics.doc/IDEAL_ASCENT_RATIO.md")]
    pub const IDEAL_ASCENT_RATIO: f32 = 0.8;
    #[doc = include_str!("line_metrics.doc/IDEAL_DESCENT_RATIO.md")]
    pub const IDEAL_DESCENT_RATIO: f32 = 0.2;

    #[doc = include_str!("line_metrics.doc/new.md")]
    #[must_use]
    pub const fn new(ascent: f32, descent: f32, line_gap: f32) -> Self {
        Self {
            ascent,
            descent,
            line_gap,
        }
    }

    #[doc = include_str!("line_metrics.doc/idealized.md")]
    #[must_use]
    pub fn idealized(font_size: f32) -> Self {
        Self::new(
            font_size * Self::IDEAL_ASCENT_RATIO,
            font_size * Self::IDEAL_DESCENT_RATIO,
            0.0,
        )
    }

    #[doc = include_str!("line_metrics.doc/ascent_2.md")]
    #[must_use]
    pub const fn ascent(&self) -> f32 {
        self.ascent
    }

    #[doc = include_str!("line_metrics.doc/descent_2.md")]
    #[must_use]
    pub const fn descent(&self) -> f32 {
        self.descent
    }

    #[doc = include_str!("line_metrics.doc/line_gap_2.md")]
    #[must_use]
    pub const fn line_gap(&self) -> f32 {
        self.line_gap
    }

    #[doc = include_str!("line_metrics.doc/line_height.md")]
    #[must_use]
    pub fn line_height(&self) -> f32 {
        self.ascent + self.descent + self.line_gap
    }

    #[doc = include_str!("line_metrics.doc/baseline.md")]
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
