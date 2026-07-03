//! The outcome of a shell open request (spec §6.11 — shell integration).

/// The outcome a [`ShellProvider`](crate::shell::ShellProvider) returns for a
/// [`ShellTarget`](crate::shell::ShellTarget) (spec §6.11).
///
/// The four outcomes mirror the spec's vocabulary: the target was opened, was
/// denied, is unsupported, or failed. [`Denied`](Self::Denied) and
/// [`Unsupported`](Self::Unsupported) are *recoverable* — the app can fall back to
/// another strategy — whereas [`Failed`](Self::Failed) is terminal and carries a
/// human-readable reason.
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

    /// Returns whether the outcome is recoverable — the app may fall back to
    /// another strategy (spec §6.11).
    ///
    /// [`Denied`](Self::Denied) and [`Unsupported`](Self::Unsupported) are
    /// recoverable; [`Failed`](Self::Failed) is terminal and
    /// [`Opened`](Self::Opened) needs no recovery.
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
