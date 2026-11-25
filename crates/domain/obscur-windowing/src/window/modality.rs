#[doc = include_str!("../../docs/api/window/window_relation.doc/Modality.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modality {
    /// The window does not block interaction with any other window.
    Modeless,
    /// The window blocks interaction with its owner window only.
    WindowModal,
    /// The window blocks interaction with every other window in the application.
    ApplicationModal,
}

impl Modality {
    /// Returns whether this modality blocks interaction with other windows.
    #[must_use]
    pub const fn is_modal(self) -> bool {
        !matches!(self, Self::Modeless)
    }
}

impl Default for Modality {
    /// The default modality is [`Modality::Modeless`].
    fn default() -> Self {
        Self::Modeless
    }
}
