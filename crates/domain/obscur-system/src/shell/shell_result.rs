#![doc = include_str!("shell_result.doc/_module.md")]

#[doc = include_str!("shell_result.doc/ShellResult.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellResult {
    #[doc = include_str!("shell_result.doc/Opened.md")]
    Opened,
    #[doc = include_str!("shell_result.doc/Denied.md")]
    Denied,
    #[doc = include_str!("shell_result.doc/Unsupported.md")]
    Unsupported,
    #[doc = include_str!("shell_result.doc/Failed.md")]
    Failed(Box<str>),
}

impl ShellResult {
    #[doc = include_str!("shell_result.doc/is_opened.md")]
    #[must_use]
    pub const fn is_opened(&self) -> bool {
        matches!(self, Self::Opened)
    }

    #[doc = include_str!("shell_result.doc/is_recoverable.md")]
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(self, Self::Denied | Self::Unsupported)
    }

    #[doc = include_str!("shell_result.doc/failure_reason.md")]
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
