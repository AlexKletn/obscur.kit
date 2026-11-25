//! Placement of title bar controls.

use super::WindowButton;

/// Ordered groups on the physical left and right of a title bar. Each list follows visual order from left to right. An empty list explicitly clears a group. An absent list leaves the group unspecified.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WindowButtonLayout {
    /// Buttons in the left group, in visual order.
    pub left: Option<Vec<WindowButton>>,
    /// Buttons in the right group, in visual order.
    pub right: Option<Vec<WindowButton>>,
}
