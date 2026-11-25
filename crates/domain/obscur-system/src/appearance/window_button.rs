//! Semantic roles for title bar controls.

/// A title bar control or spacer. The host determines which actions the current window supports. The maximize control toggles between maximization and restoration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum WindowButton {
    /// Opens the window actions menu.
    WindowMenu,
    /// Opens the application menu.
    ApplicationMenu,
    /// Displays the application icon in the title bar.
    WindowIcon,
    /// Minimizes the window.
    Minimize,
    /// Toggles between maximized and restored size.
    Maximize,
    /// Closes the window.
    Close,
    /// Enters context help mode.
    ContextHelp,
    /// Toggles whether only the title bar is shown.
    Shade,
    /// Toggles whether the window stays above others.
    KeepAbove,
    /// Toggles whether the window stays below others.
    KeepBelow,
    /// Toggles visibility across workspaces.
    OnAllWorkspaces,
    /// Toggles exclusion from screenshots and screen recordings.
    ExcludeFromCapture,
    /// Adds space between controls.
    Spacer,
}
