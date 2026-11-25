//! Events from a native layer popup.

use obscur_geometry::Rect;
use obscur_interaction::command::{KeyEvent, Modifiers};
use obscur_interaction::pointer::{PointerEvent, WheelEvent};

/// Events from a native layer popup.
#[derive(Debug, Clone)]
pub enum LayerPopupEvent {
    /// The compositor-selected rectangle relative to the panel, in logical pixels.
    Configured(Rect),
    /// Input in the popup surface’s local logical coordinates.
    Pointer(PointerEvent),
    /// Wheel deltas in logical pixels relative to the popup surface.
    Wheel(WheelEvent),
    /// A key delivered while the native popup has keyboard focus.
    Key(KeyEvent),
    /// Current modifiers for the popup keyboard.
    Modifiers(Modifiers),
    /// Whether the adapter is routing native keyboard input to the popup.
    KeyboardFocus(bool),
    /// The popup has closed through a client request or compositor dismissal.
    Closed,
    /// The popup could not be opened or presented.
    Failed(Box<str>),
}
