/// Motion of a background image relative to scrolling.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundAttachment {
    /// Moves with the box.
    #[default]
    Scroll,
    /// Moves with the box's scrollable contents.
    Local,
    /// Stays attached to the viewport.
    Fixed,
}
