//! Presentation mode saved with window placement.

use serde::{Deserialize, Serialize};

/// The window presentation mode restored at startup.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowMode {
    #[default]
    /// A regular window.
    Normal,
    /// A maximized window.
    Maximized,
    /// Borderless fullscreen on the selected monitor.
    Fullscreen,
}
