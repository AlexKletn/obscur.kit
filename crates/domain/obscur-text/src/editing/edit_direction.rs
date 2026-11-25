#[doc = include_str!("../../docs/api/editing/edit_command.doc/EditDirection.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditDirection {
    /// Toward the start of the buffer (left / up / Backspace).
    Backward,
    /// Toward the end of the buffer (right / down / Delete).
    Forward,
}
