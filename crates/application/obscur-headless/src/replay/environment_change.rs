/// A recorded window/environment change (spec §7.13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnvironmentChange {
    /// The window was resized to `(width, height)` in physical pixels.
    WindowResized {
        /// New window width.
        width: u32,
        /// New window height.
        height: u32,
    },
    /// The window's visibility changed (`true` = visible).
    VisibilityChanged {
        /// Whether the window is now visible.
        visible: bool,
    },
}
