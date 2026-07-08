//! A style-local `linear-gradient(...)` value (spec §5.2).

use crate::property::{GradientStop, Length};

/// One of the four box corners a `linear-gradient(to <corner>, …)` can point at.
///
/// Unlike a `to <side>` keyword (which folds to a fixed angle), a `to <corner>`
/// direction is **aspect-ratio-aware**: per [CSS Images §3.1] the gradient line is
/// angled so it points into the corner's quadrant and is *perpendicular to the
/// diagonal joining the two neighbouring corners* of the box. That angle depends
/// on the box's width/height, which is unknown at CSS-lowering time, so the corner
/// is carried symbolically here and resolved against the painted box by the scene
/// coordinator.
///
/// [CSS Images §3.1]: https://drafts.csswg.org/css-images-3/#linear-gradients
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    /// `to top left`.
    TopLeft,
    /// `to top right`.
    TopRight,
    /// `to bottom right`.
    BottomRight,
    /// `to bottom left`.
    BottomLeft,
}

/// A CSS `linear-gradient(...)` image, as authored in a stylesheet.
///
/// The gradient is described by an `angle_deg` on the CSS convention — `0` points
/// *to top*, `90` *to right*, `180` *to bottom*, `270` *to left*, increasing
/// clockwise — and an ordered list of color [`GradientStop`]s. The `to <side>`
/// keyword forms are folded to their equivalent angle by the CSS compiler, so
/// this type stores only an angle.
///
/// The angle defines the gradient *line*; the scene coordinator turns it plus the
/// painted box's bounds into the concrete start/end points the renderer's
/// `obscur_scene::paint::LinearGradient` needs. This type is a pure style concept
/// and deliberately has **no dependency on the scene**.
///
/// # Repeating gradients
///
/// `repeating-linear-gradient(...)` is modelled as this same value with
/// `repeating` set and a `period` carried. A non-repeating gradient spans the
/// whole box (offset `0`→`1` across the painted box); a repeating one instead
/// defines *one period* — the extent from the last authored color-stop position —
/// and the pattern tiles across the box. The [`stops`](Self::stops) are always
/// normalized to `0.0..=1.0` **over that period** (the CSS lowering divides each
/// authored position by the period), and `period` records the period's length so
/// the scene coordinator can size the gradient axis to exactly one period and let
/// the renderer tile it with `ExtendMode::Repeat`:
///
/// - [`Length::Px`] — an absolute px-positioned period (the common stripe case,
///   e.g. `repeating-linear-gradient(45deg, #000 0 8px, #fff 8px 16px)` = a 16px
///   period);
/// - [`Length::Percent`] — a period expressed as a fraction of the gradient line.
///
/// Because [`GradientStop`] carries `f32` data, this type derives [`PartialEq`]
/// but not [`Eq`]/[`Hash`].
#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    /// The gradient line's angle in degrees (CSS convention: `0` = to top,
    /// increasing clockwise). When [`corner`](Self::corner) is `Some`, this holds
    /// the nominal diagonal angle (45°/135°/…) as a fallback, but the scene
    /// coordinator computes the true aspect-ratio-aware angle from the box instead.
    pub angle_deg: f32,
    /// A `to <corner>` direction, resolved aspect-ratio-aware at paint time
    /// ([CSS Images §3.1]). `None` for an explicit `<angle>` or a `to <side>` form,
    /// where [`angle_deg`](Self::angle_deg) fully defines the gradient line.
    ///
    /// [CSS Images §3.1]: https://drafts.csswg.org/css-images-3/#linear-gradients
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
        }
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

    /// Returns the tiling period of a repeating gradient, or [`None`] for a
    /// non-repeating one.
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
