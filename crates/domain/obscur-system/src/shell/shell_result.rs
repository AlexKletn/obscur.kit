//! The outcome of a shell open request (spec §6.11 — shell integration).

#[doc = include_str!("../../docs/api/shell/shell_result.doc/ShellResult.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellResult {
    /// The shell opened the target.
    Opened,
    /// Opening was denied by permission policy.
    Denied,
    /// The provider does not support this kind of target.
    Unsupported,
    /// Opening was attempted but failed, carrying a human-readable reason.
    Failed(Box<str>),
}

impl ShellResult {
    /// Returns whether the target was opened (spec §6.11).
    #[must_use]
    pub const fn is_opened(&self) -> bool {
        matches!(self, Self::Opened)
    }

    #[doc = include_str!("../../docs/api/shell/shell_result.doc/is_recoverable.md")]
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(self, Self::Denied | Self::Unsupported)
    }

    /// Returns the failure reason when the outcome is [`Failed`](Self::Failed),
    /// else `None` (spec §6.11).
    #[must_use]
    pub fn failure_reason(&self) -> Option<&str> {
        match self {
            Self::Failed(reason) => Some(reason),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_opened_only_for_opened() {
        assert!(ShellResult::Opened.is_opened());
        assert!(!ShellResult::Denied.is_opened());
        assert!(!ShellResult::Failed(Box::from("x")).is_opened());
    }

    #[test]
    fn recoverable_covers_denied_and_unsupported_only() {
        assert!(ShellResult::Denied.is_recoverable());
        assert!(ShellResult::Unsupported.is_recoverable());
        assert!(!ShellResult::Opened.is_recoverable());
        assert!(!ShellResult::Failed(Box::from("boom")).is_recoverable());
    }

    #[test]
    fn failure_reason_only_for_failed() {
        assert_eq!(
            ShellResult::Failed(Box::from("no handler")).failure_reason(),
            Some("no handler")
        );
        assert_eq!(ShellResult::Opened.failure_reason(), None);
        assert_eq!(ShellResult::Unsupported.failure_reason(), None);
    }
}
