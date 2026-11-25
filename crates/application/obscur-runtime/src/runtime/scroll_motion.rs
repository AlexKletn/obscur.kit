//! Time-based interpolation for a programmatic scroll.

use obscur_geometry::Point;

/// A two-axis scroll interpolation on the runtime motion clock.
#[derive(Clone, Copy)]
pub(super) struct ScrollMotion {
    start: Point,
    end: Point,
    duration: f32,
    elapsed: f32,
}

impl ScrollMotion {
    /// Creates a scroll interpolation from the live offset to the destination.
    pub(super) fn new(start: Point, end: Point, duration: f32) -> Self {
        Self {
            start,
            end,
            duration,
            elapsed: 0.0,
        }
    }

    /// Samples the offset after advancing by seconds and reports whether motion remains.
    pub(super) fn advance(&mut self, delta: f32) -> (Point, bool) {
        self.elapsed += delta;
        if self.elapsed >= self.duration {
            return (self.end, false);
        }
        let remaining = 1.0 - self.elapsed / self.duration;
        let progress = 1.0 - remaining.powi(3);
        let interpolate = |start, end| start + (end - start) * progress;
        (
            Point::new(
                interpolate(self.start.x, self.end.x),
                interpolate(self.start.y, self.end.y),
            ),
            true,
        )
    }
}
