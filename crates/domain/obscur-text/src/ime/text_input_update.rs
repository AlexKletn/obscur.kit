//! An atomic update from a native text input session.

use super::Preedit;

/// An atomic update from a native text input session.
#[derive(Debug, Default, Clone)]
pub struct TextInputUpdate {
    /// Bytes to delete before the selection.
    pub before: u32,
    /// Bytes to delete after the selection.
    pub after: u32,
    /// Text committed in this update.
    pub commit: Option<Box<str>>,
    /// Replacement preedit for this update.
    pub preedit: Option<Preedit>,
}
