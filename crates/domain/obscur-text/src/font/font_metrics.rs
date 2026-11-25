#[doc = include_str!("../../docs/api/font/font_face.doc/FontMetrics.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontMetrics {
    /// The number of design units per em; the scale for every other field.
    pub units_per_em: u16,
    /// The ascent above the baseline, in font units (typically positive).
    pub ascent: f32,
    /// The descent below the baseline, in font units (typically negative).
    pub descent: f32,
    /// Extra leading between lines, in font units.
    pub line_gap: f32,
    /// The height of a flat capital, in font units, if the font reports it.
    pub cap_height: Option<f32>,
    /// The height of a lowercase `x`, in font units, if the font reports it.
    pub x_height: Option<f32>,
}

impl FontMetrics {
    /// Creates metrics from the required fields, leaving
    /// [`cap_height`](Self::cap_height) and [`x_height`](Self::x_height) unset.
    #[must_use]
    pub const fn new(units_per_em: u16, ascent: f32, descent: f32, line_gap: f32) -> Self {
        Self {
            units_per_em,
            ascent,
            descent,
            line_gap,
            cap_height: None,
            x_height: None,
        }
    }

    /// Returns these metrics with [`cap_height`](Self::cap_height) set.
    #[must_use]
    pub const fn with_cap_height(mut self, cap_height: f32) -> Self {
        self.cap_height = Some(cap_height);
        self
    }

    /// Returns these metrics with [`x_height`](Self::x_height) set.
    #[must_use]
    pub const fn with_x_height(mut self, x_height: f32) -> Self {
        self.x_height = Some(x_height);
        self
    }

    #[doc = include_str!("../../docs/api/font/font_face.doc/line_height.md")]
    #[must_use]
    pub fn line_height(&self) -> f32 {
        self.ascent + self.descent.abs() + self.line_gap
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_height_uses_descent_magnitude() {
        // Descent given as a negative value per font conventions.
        let metrics = FontMetrics::new(1000, 800.0, -200.0, 100.0);
        assert_eq!(metrics.line_height(), 1100.0);
        // A positive descent yields the same magnitude-based result.
        let positive = FontMetrics::new(1000, 800.0, 200.0, 100.0);
        assert_eq!(positive.line_height(), 1100.0);
    }

    #[test]
    fn optional_metrics_default_none_then_set() {
        let bare = FontMetrics::new(2048, 1500.0, -400.0, 0.0);
        assert_eq!(bare.cap_height, None);
        assert_eq!(bare.x_height, None);
        let full = bare.with_cap_height(1400.0).with_x_height(1000.0);
        assert_eq!(full.cap_height, Some(1400.0));
        assert_eq!(full.x_height, Some(1000.0));
    }
}
