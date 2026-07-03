//! Edge autoscroll configuration and computation during a drag (spec §6.21 drag
//! and drop — autoscroll).

use obscur_geometry::{Point, Size};

/// Edge autoscroll configuration and per-tick velocity computation for a drag
/// (spec §6.21 autoscroll).
///
/// When a drag's pointer nears a viewport edge the surface should scroll to
/// reveal more content. This value configures the [`edge_margin`] — the
/// distance from an edge within which autoscroll engages — and the
/// [`max_velocity`] reached at the very edge, and computes a deterministic
/// per-tick [`velocity`] from the pointer position and viewport size. Velocity
/// scales linearly with how deep into the margin the pointer sits and is
/// negative toward the low edge, positive toward the high edge.
///
/// [`edge_margin`]: DragAutoscroll::edge_margin
/// [`max_velocity`]: DragAutoscroll::max_velocity
/// [`velocity`]: DragAutoscroll::velocity
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragAutoscroll {
    edge_margin: f32,
    max_velocity: f32,
}

impl DragAutoscroll {
    /// Creates an autoscroll config with the given `edge_margin` (px) and
    /// `max_velocity` (px/tick) (spec §6.21).
    #[must_use]
    pub const fn new(edge_margin: f32, max_velocity: f32) -> Self {
        Self {
            edge_margin,
            max_velocity,
        }
    }

    /// Returns the edge margin in pixels within which autoscroll engages (spec
    /// §6.21).
    #[must_use]
    pub const fn edge_margin(&self) -> f32 {
        self.edge_margin
    }

    /// Returns the maximum velocity in px/tick reached at the very edge (spec
    /// §6.21).
    #[must_use]
    pub const fn max_velocity(&self) -> f32 {
        self.max_velocity
    }

    /// Computes the deterministic per-tick scroll velocity `(vx, vy)` for the
    /// pointer at `pointer` within `viewport` (spec §6.21).
    ///
    /// For each axis: a pointer within [`edge_margin`](Self::edge_margin) of the
    /// low edge yields a negative velocity, within the margin of the high edge a
    /// positive velocity, and zero otherwise. The magnitude scales linearly with
    /// margin depth, clamped to `[0, 1]` of [`max_velocity`](Self::max_velocity).
    #[must_use]
    pub fn velocity(&self, pointer: Point, viewport: Size) -> (f32, f32) {
        (
            self.axis_velocity(pointer.x, viewport.width),
            self.axis_velocity(pointer.y, viewport.height),
        )
    }

    /// Returns whether either axis would autoscroll for this pointer (spec
    /// §6.21).
    #[must_use]
    pub fn is_scrolling(&self, pointer: Point, viewport: Size) -> bool {
        let (vx, vy) = self.velocity(pointer, viewport);
        vx != 0.0 || vy != 0.0
    }

    /// Computes the signed velocity along one axis for a pointer coordinate
    /// `pos` within an extent `extent`.
    fn axis_velocity(self, pos: f32, extent: f32) -> f32 {
        if pos < self.edge_margin {
            let depth = (self.edge_margin - pos) / self.edge_margin;
            -self.max_velocity * depth.clamp(0.0, 1.0)
        } else if pos > extent - self.edge_margin {
            let depth = (pos - (extent - self.edge_margin)) / self.edge_margin;
            self.max_velocity * depth.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

impl Default for DragAutoscroll {
    /// A 32 px edge margin with a 20 px/tick maximum velocity (spec §6.21).
    fn default() -> Self {
        Self::new(32.0, 20.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_margin_and_velocity() {
        let auto = DragAutoscroll::default();
        assert!((auto.edge_margin() - 32.0).abs() < 1e-6);
        assert!((auto.max_velocity() - 20.0).abs() < 1e-6);
    }

    #[test]
    fn left_edge_is_negative() {
        let auto = DragAutoscroll::default();
        let (vx, vy) = auto.velocity(Point::new(0.0, 100.0), Size::new(400.0, 400.0));
        assert!(vx < 0.0);
        assert!((vx - (-20.0)).abs() < 1e-6); // depth = 1.0 at pos 0
        assert!(vy.abs() < 1e-6);
    }

    #[test]
    fn right_edge_is_positive() {
        let auto = DragAutoscroll::default();
        let (vx, vy) = auto.velocity(Point::new(400.0, 100.0), Size::new(400.0, 400.0));
        assert!(vx > 0.0);
        assert!((vx - 20.0).abs() < 1e-6); // depth = 1.0 at the far edge
        assert!(vy.abs() < 1e-6);
    }

    #[test]
    fn center_is_zero() {
        let auto = DragAutoscroll::default();
        let (vx, vy) = auto.velocity(Point::new(200.0, 200.0), Size::new(400.0, 400.0));
        assert!(vx.abs() < 1e-6);
        assert!(vy.abs() < 1e-6);
        assert!(!auto.is_scrolling(Point::new(200.0, 200.0), Size::new(400.0, 400.0)));
    }

    #[test]
    fn corner_scrolls_both_axes() {
        let auto = DragAutoscroll::default();
        let (vx, vy) = auto.velocity(Point::new(0.0, 0.0), Size::new(400.0, 400.0));
        assert!(vx < 0.0);
        assert!(vy < 0.0);
        assert!(auto.is_scrolling(Point::new(0.0, 0.0), Size::new(400.0, 400.0)));
    }

    #[test]
    fn velocity_scales_with_margin_depth() {
        let auto = DragAutoscroll::default();
        // Halfway into the left margin → half of max velocity.
        let (vx, _) = auto.velocity(Point::new(16.0, 200.0), Size::new(400.0, 400.0));
        assert!((vx - (-10.0)).abs() < 1e-6);
    }
}
