#![doc = include_str!("window_constraints.doc/_module.md")]

use obscur_geometry::Size;

#[doc = include_str!("window_constraints.doc/WindowConstraints.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WindowConstraints {
    #[doc = include_str!("window_constraints.doc/min_size.md")]
    pub min_size: Option<Size>,
    #[doc = include_str!("window_constraints.doc/max_size.md")]
    pub max_size: Option<Size>,
}

impl WindowConstraints {
    #[doc = include_str!("window_constraints.doc/UNCONSTRAINED.md")]
    pub const UNCONSTRAINED: Self = Self {
        min_size: None,
        max_size: None,
    };

    #[doc = include_str!("window_constraints.doc/new.md")]
    #[must_use]
    pub const fn new(min_size: Option<Size>, max_size: Option<Size>) -> Self {
        Self { min_size, max_size }
    }

    #[doc = include_str!("window_constraints.doc/with_min.md")]
    #[must_use]
    pub const fn with_min(min_size: Size) -> Self {
        Self {
            min_size: Some(min_size),
            max_size: None,
        }
    }

    #[doc = include_str!("window_constraints.doc/with_max.md")]
    #[must_use]
    pub const fn with_max(max_size: Size) -> Self {
        Self {
            min_size: None,
            max_size: Some(max_size),
        }
    }

    #[doc = include_str!("window_constraints.doc/constrain.md")]
    #[must_use]
    pub fn constrain(&self, size: Size) -> Size {
        Size::new(
            clamp_axis(
                size.width,
                self.min_size.map(|s| s.width),
                self.max_size.map(|s| s.width),
            ),
            clamp_axis(
                size.height,
                self.min_size.map(|s| s.height),
                self.max_size.map(|s| s.height),
            ),
        )
    }
}

#[doc = include_str!("window_constraints.doc/clamp_axis.md")]
fn clamp_axis(value: f32, min: Option<f32>, max: Option<f32>) -> f32 {
    let mut v = value;
    if let Some(max) = max
        && v > max
    {
        v = max;
    }
    if let Some(min) = min
        && v < min
    {
        v = min;
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_unconstrained() {
        assert_eq!(
            WindowConstraints::default(),
            WindowConstraints::UNCONSTRAINED
        );
        let c = WindowConstraints::default();
        let s = Size::new(123.0, 456.0);
        assert_eq!(c.constrain(s), s);
    }

    #[test]
    fn clamps_below_min() {
        let c = WindowConstraints::with_min(Size::new(100.0, 200.0));
        assert_eq!(c.constrain(Size::new(10.0, 10.0)), Size::new(100.0, 200.0));
    }

    #[test]
    fn clamps_above_max() {
        let c = WindowConstraints::with_max(Size::new(800.0, 600.0));
        assert_eq!(
            c.constrain(Size::new(1000.0, 1000.0)),
            Size::new(800.0, 600.0)
        );
    }

    #[test]
    fn passes_through_within_range() {
        let c =
            WindowConstraints::new(Some(Size::new(100.0, 100.0)), Some(Size::new(800.0, 600.0)));
        let s = Size::new(400.0, 300.0);
        assert_eq!(c.constrain(s), s);
    }

    #[test]
    fn min_wins_when_greater_than_max() {
        let c =
            WindowConstraints::new(Some(Size::new(500.0, 500.0)), Some(Size::new(100.0, 100.0)));
        // min (500) exceeds max (100); the minimum must win.
        assert_eq!(
            c.constrain(Size::new(300.0, 300.0)),
            Size::new(500.0, 500.0)
        );
    }
}
