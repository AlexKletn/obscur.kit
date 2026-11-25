#[doc = include_str!("../../docs/api/dialog/dialog_request.doc/MessageLevel.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    /// A neutral, informational message.
    Info,
    /// A cautionary message the user should notice but that is not an error.
    Warning,
    /// An error message reporting that something went wrong.
    Error,
}
