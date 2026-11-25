//! Theme icon identifiers for common title bar controls.

/// Optional icon names to look up in the host icon theme. The host can supply a fallback glyph when an icon is unavailable.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WindowButtonIcons {
    /// Theme icon for minimizing.
    pub minimize: Option<Box<str>>,
    /// Theme icon for maximizing.
    pub maximize: Option<Box<str>>,
    /// Theme icon for restoring a maximized window.
    pub restore: Option<Box<str>>,
    /// Theme icon for closing.
    pub close: Option<Box<str>>,
}
