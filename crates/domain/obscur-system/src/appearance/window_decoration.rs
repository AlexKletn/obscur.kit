//! Appearance of a window title bar.

use super::{WindowButtonIcons, WindowButtonLayout};

/// Desktop preferences for title bar controls. A provider supplies layout and icon identifiers together so consumers can use them for client-side decorations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowDecoration {
    /// Placement and order of the configured title bar controls.
    pub button_layout: WindowButtonLayout,
    /// Theme icon identifiers for common actions.
    pub button_icons: WindowButtonIcons,
}
