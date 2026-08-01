#![doc = include_str!("window_operation_result.doc/_module.md")]

use obscur_capability::SupportLevel;

#[doc = include_str!("window_operation_result.doc/WindowOperationResult.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowOperationResult {
    #[doc = include_str!("window_operation_result.doc/Performed.md")]
    Performed(SupportLevel),
    #[doc = include_str!("window_operation_result.doc/Unsupported.md")]
    Unsupported,
    #[doc = include_str!("window_operation_result.doc/Failed.md")]
    Failed(Box<str>),
}

impl WindowOperationResult {
    #[doc = include_str!("window_operation_result.doc/native.md")]
    #[must_use]
    pub const fn native() -> Self {
        Self::Performed(SupportLevel::Native)
    }

    #[doc = include_str!("window_operation_result.doc/emulated.md")]
    #[must_use]
    pub const fn emulated() -> Self {
        Self::Performed(SupportLevel::Emulated)
    }

    #[doc = include_str!("window_operation_result.doc/conditional.md")]
    #[must_use]
    pub const fn conditional() -> Self {
        Self::Performed(SupportLevel::Limited)
    }

    #[doc = include_str!("window_operation_result.doc/failed.md")]
    #[must_use]
    pub fn failed(reason: impl Into<Box<str>>) -> Self {
        Self::Failed(reason.into())
    }

    #[doc = include_str!("window_operation_result.doc/is_success.md")]
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Performed(_))
    }

    #[doc = include_str!("window_operation_result.doc/is_unsupported.md")]
    #[must_use]
    pub const fn is_unsupported(&self) -> bool {
        matches!(self, Self::Unsupported)
    }

    #[doc = include_str!("window_operation_result.doc/support_level.md")]
    #[must_use]
    pub const fn support_level(&self) -> Option<SupportLevel> {
        match self {
            Self::Performed(level) => Some(*level),
            Self::Unsupported => Some(SupportLevel::Unavailable),
            Self::Failed(_) => None,
        }
    }

    #[doc = include_str!("window_operation_result.doc/failure_reason.md")]
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
    fn performed_constructors_carry_levels() {
        assert_eq!(
            WindowOperationResult::native(),
            WindowOperationResult::Performed(SupportLevel::Native)
        );
        assert_eq!(
            WindowOperationResult::emulated(),
            WindowOperationResult::Performed(SupportLevel::Emulated)
        );
        assert_eq!(
            WindowOperationResult::conditional(),
            WindowOperationResult::Performed(SupportLevel::Limited)
        );
    }

    #[test]
    fn is_success_only_for_performed() {
        assert!(WindowOperationResult::native().is_success());
        assert!(WindowOperationResult::emulated().is_success());
        assert!(WindowOperationResult::conditional().is_success());
        assert!(!WindowOperationResult::Unsupported.is_success());
        assert!(!WindowOperationResult::failed("boom").is_success());
    }

    #[test]
    fn is_unsupported_predicate() {
        assert!(WindowOperationResult::Unsupported.is_unsupported());
        assert!(!WindowOperationResult::native().is_unsupported());
        assert!(!WindowOperationResult::failed("boom").is_unsupported());
    }

    #[test]
    fn support_level_mapping() {
        assert_eq!(
            WindowOperationResult::native().support_level(),
            Some(SupportLevel::Native)
        );
        assert_eq!(
            WindowOperationResult::conditional().support_level(),
            Some(SupportLevel::Limited)
        );
        assert_eq!(
            WindowOperationResult::Unsupported.support_level(),
            Some(SupportLevel::Unavailable)
        );
        assert_eq!(WindowOperationResult::failed("boom").support_level(), None);
    }

    #[test]
    fn failure_reason_only_for_failed() {
        assert_eq!(
            WindowOperationResult::failed("no wayland").failure_reason(),
            Some("no wayland")
        );
        assert_eq!(WindowOperationResult::native().failure_reason(), None);
        assert_eq!(WindowOperationResult::Unsupported.failure_reason(), None);
    }
}
