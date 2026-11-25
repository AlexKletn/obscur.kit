//! Platform-reported state of an application window.

use crate::geometry::Size;

/// A snapshot of the ordinary desktop window as last reported by the window backend.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowState {
    /// The window content size in logical pixels. During a live resize this may be newer than the viewport of the frame being drawn.
    pub size: Size,
    /// Device pixels per logical pixel.
    pub scale_factor: f64,
    /// Whether the window is maximized.
    pub maximized: bool,
    /// Whether the window is fullscreen.
    pub fullscreen: bool,
    /// Whether the window is minimized, if the backend can report it. Wayland reports `None`.
    pub minimized: Option<bool>,
    /// Whether the window has keyboard focus.
    pub focused: bool,
    /// Whether the backend allows the window to be resized by the user.
    pub resizable: bool,
}
