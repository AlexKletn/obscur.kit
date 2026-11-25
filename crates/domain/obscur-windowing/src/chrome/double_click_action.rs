#[doc = include_str!("../../docs/api/chrome/drag_region.doc/DoubleClickAction.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DoubleClickAction {
    /// Double-clicking does nothing.
    None,
    /// Double-clicking toggles the window maximized (the conventional action on
    /// most platforms).
    Maximize,
    /// Double-clicking minimizes the window.
    Minimize,
}

impl Default for DoubleClickAction {
    /// The default action is [`DoubleClickAction::None`].
    fn default() -> Self {
        Self::None
    }
}
