/// A swipe direction in logical window coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwipeDirection {
    /// Movement to the left.
    Left,
    /// Movement to the right.
    Right,
    /// Movement toward the top of the window.
    Up,
    /// Movement toward the bottom of the window.
    Down,
}
