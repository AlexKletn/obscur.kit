//! A style-local `linear-gradient(...)` value (spec §5.2).

use crate::property::{GradientStop, Length};

use super::Corner;

#[doc = include_str!("../../docs/api/property/linear_gradient.doc/LinearGradient.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    /// The gradient line's angle in degrees (CSS convention: `0` = to top,
    /// increasing clockwise). When [`corner`](Self::corner) is `Some`, this holds
    /// the nominal diagonal angle (45°/135°/…) as a fallback, but the scene
    /// coordinator computes the true aspect-ratio-aware angle from the box instead.
    pub angle_deg: f32,
    #[doc = include_str!("../../docs/api/property/linear_gradient.doc/corner.md")]
    pub corner: Option<Corner>,
    /// The gradient's color stops, in authored order along the line. For a
    /// repeating gradient these are normalized to `0.0..=1.0` over one `period`.
    pub stops: Box<[GradientStop]>,
    /// Whether this is a `repeating-linear-gradient` (tiles one `period` across
    /// the box). `false` for the ordinary box-spanning `linear-gradient`.
    pub repeating: bool,
    /// The tiling period of a repeating gradient, or [`None`] for a
    /// non-repeating one. `Some` iff `repeating` is `true`.
    pub period: Option<Length>,
    /// Authored positions for repeats whose period or start depend on the gradient box.
    repeat_positions: Option<Box<[Option<Length>]>>,
}

impl LinearGradient {
    /// Creates a non-repeating linear gradient at `angle_deg` (CSS convention)
    /// with `stops`, spanning the whole box.
    #[must_use]
    pub fn new(angle_deg: f32, stops: Vec<GradientStop>) -> Self {
        Self {
            angle_deg,
            corner: None,
            stops: stops.into_boxed_slice(),
            repeating: false,
            period: None,
            repeat_positions: None,
        }
    }

    /// Creates a non-repeating `to <corner>` linear gradient. The `angle_deg` is
    /// the nominal diagonal angle (a fallback); the scene coordinator resolves the
    /// aspect-ratio-aware angle from the box (spec §3.1).
    #[must_use]
    pub fn new_corner(corner: Corner, angle_deg: f32, stops: Vec<GradientStop>) -> Self {
        Self {
            angle_deg,
            corner: Some(corner),
            stops: stops.into_boxed_slice(),
            repeating: false,
            period: None,
            repeat_positions: None,
        }
    }

    /// Sets the `to <corner>` direction, returning the updated gradient (used to
    /// attach a corner to a repeating gradient).
    #[must_use]
    pub fn with_corner(mut self, corner: Corner) -> Self {
        self.corner = Some(corner);
        self
    }

    /// Creates a repeating linear gradient at `angle_deg` (CSS convention) whose
    /// one-`period` pattern tiles across the box. The `stops` positions are the
    /// per-period normalized `0.0..=1.0` values.
    #[must_use]
    pub fn new_repeating(angle_deg: f32, stops: Vec<GradientStop>, period: Length) -> Self {
        Self {
            angle_deg,
            corner: None,
            stops: stops.into_boxed_slice(),
            repeating: true,
            period: Some(period),
            repeat_positions: None,
        }
    }

    /// Associates each stop with its authored length, percentage, or automatic position.
    #[must_use]
    pub fn with_repeat_positions(mut self, positions: Box<[Option<Length>]>) -> Self {
        if positions.len() == self.stops.len() && positions.len() >= 2 {
            self.period = None;
            self.repeating = true;
            self.repeat_positions = Some(positions);
        }
        self
    }

    /// Returns authored stop positions for a repeat that is resolved at paint time.
    #[must_use]
    pub fn repeat_positions(&self) -> Option<&[Option<Length>]> {
        self.repeat_positions
            .as_deref()
            .filter(|positions| positions.len() == self.stops.len())
    }

    /// Whether any repeat distance uses ch units.
    #[must_use]
    pub fn uses_ch(&self) -> bool {
        self.period.is_some_and(Length::is_ch)
            || self
                .repeat_positions()
                .is_some_and(|positions| positions.iter().flatten().any(|length| length.is_ch()))
    }

    /// Returns the gradient line's angle in degrees (CSS convention).
    #[must_use]
    pub const fn angle_deg(&self) -> f32 {
        self.angle_deg
    }

    /// Returns the `to <corner>` direction, or `None` for an angle / side form.
    #[must_use]
    pub const fn corner(&self) -> Option<Corner> {
        self.corner
    }

    /// Returns the gradient's color stops.
    #[must_use]
    pub const fn stops(&self) -> &[GradientStop] {
        &self.stops
    }

    /// Returns whether this is a repeating gradient (tiles one period).
    #[must_use]
    pub const fn repeating(&self) -> bool {
        self.repeating
    }

    #[doc = include_str!("../../docs/api/property/linear_gradient.doc/period_2.md")]
    #[must_use]
    pub const fn period(&self) -> Option<Length> {
        self.period
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::StyleColor;

    #[test]
    fn new_stores_angle_and_stops() {
        let g = LinearGradient::new(
            90.0,
            vec![
                GradientStop::new(StyleColor::BLACK, 0.0),
                GradientStop::new(StyleColor::rgba(1.0, 1.0, 1.0, 1.0), 1.0),
            ],
        );
        assert_eq!(g.angle_deg(), 90.0);
        assert_eq!(g.stops().len(), 2);
        assert_eq!(g.stops()[0].position(), Some(0.0));
        assert_eq!(g.stops()[1].color(), StyleColor::rgba(1.0, 1.0, 1.0, 1.0));
        // A plain gradient is non-repeating and carries no period.
        assert!(!g.repeating());
        assert_eq!(g.period(), None);
    }

    #[test]
    fn repeating_carries_period_and_flag() {
        let g = LinearGradient::new_repeating(
            90.0,
            vec![
                GradientStop::new(StyleColor::BLACK, 0.0),
                GradientStop::new(StyleColor::rgba(1.0, 1.0, 1.0, 1.0), 1.0),
            ],
            Length::px(8.0),
        );
        assert!(g.repeating());
        assert_eq!(g.period(), Some(Length::px(8.0)));
    }
}
