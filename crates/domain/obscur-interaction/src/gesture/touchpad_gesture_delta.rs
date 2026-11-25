use obscur_geometry::Point;

/// Displacement or transform supplied by a native touchpad gesture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TouchpadGestureDelta {
    /// Incremental finger displacement in logical pixels.
    Swipe(Point),
    /// Scale ratio and clockwise angle in radians measured from the start of the gesture.
    Transform {
        /// Ratio to the initial separation.
        scale: f32,
        /// Clockwise radians from the initial orientation.
        rotation: f32,
    },
}
