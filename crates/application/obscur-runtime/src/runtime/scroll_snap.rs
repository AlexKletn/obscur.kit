//! Tracks application-selected scroll snap positions and their quiet period.

use obscur_geometry::Point;

const QUIET_PERIOD: f32 = 0.12;
pub(super) const SNAP_DURATION: f32 = 0.18;

pub(super) struct ScrollSnap {
    pub(super) candidates: Vec<Point>,
    remaining: Option<f32>,
}

impl ScrollSnap {
    pub(super) fn new(candidates: Vec<Point>) -> Self {
        Self {
            candidates,
            remaining: None,
        }
    }

    pub(super) fn arm(&mut self) {
        self.remaining = Some(QUIET_PERIOD);
    }

    pub(super) fn cancel(&mut self) {
        self.remaining = None;
    }

    pub(super) fn is_pending(&self) -> bool {
        self.remaining.is_some()
    }

    pub(super) fn advance(&mut self, delta: f32) -> bool {
        let Some(remaining) = self.remaining.as_mut() else {
            return false;
        };
        *remaining -= delta;
        if *remaining > 0.0 {
            return false;
        }
        self.cancel();
        true
    }
}
