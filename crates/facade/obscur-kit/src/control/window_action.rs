use super::WindowResizeDirection;

/// An operation on the application window.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowAction {
    /// Minimize the window.
    Minimize,
    /// Switch between maximized and restored.
    ToggleMaximized,
    /// Close the window and exit its event loop.
    Close,
    /// Starts native interactive window movement from a primary mouse press.
    BeginMove,
    /// Starts native interactive resizing from a primary mouse press on the selected edge or corner.
    BeginResize(WindowResizeDirection),
}
