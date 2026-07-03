//! Typed outcome of a dynamic window property operation (spec §6.2).

use obscur_capability::SupportLevel;

/// The typed outcome of a §6.2 dynamic window property or window operation.
///
/// Spec §6.2 requires that operations return typed results across the capability
/// categories *native*, *emulated*, *conditional*, and *unsupported*. This enum
/// models the outcome an application observes after submitting such an
/// operation, reusing [`SupportLevel`] as the canonical category type: the spec
/// *conditional* category maps to [`SupportLevel::Limited`], and *unsupported*
/// maps to the distinct [`WindowOperationResult::Unsupported`] variant
/// (conceptually [`SupportLevel::Unavailable`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowOperationResult {
    /// The operation was satisfied; the [`SupportLevel`] records *how* it was
    /// satisfied per spec §6.2 (`Native`, `Emulated`, or `Limited` for the
    /// *conditional* category).
    Performed(SupportLevel),
    /// The platform does not offer the operation at all (spec §6.2
    /// *unsupported*). Distinct from a runtime [`WindowOperationResult::Failed`]:
    /// the operation was never attempted because it is unavailable.
    Unsupported,
    /// The operation was attempted but failed at runtime, carrying a
    /// human-readable reason (spec §6.2). This is not a support-level category:
    /// the platform offers the operation, but this particular attempt did not
    /// succeed.
    Failed(Box<str>),
}

impl WindowOperationResult {
    /// A [`WindowOperationResult::Performed`] with [`SupportLevel::Native`]:
    /// the operation was satisfied natively (spec §6.2).
    #[must_use]
    pub const fn native() -> Self {
        Self::Performed(SupportLevel::Native)
    }

    /// A [`WindowOperationResult::Performed`] with [`SupportLevel::Emulated`]:
    /// the operation was satisfied through emulation (spec §6.2).
    #[must_use]
    pub const fn emulated() -> Self {
        Self::Performed(SupportLevel::Emulated)
    }

    /// A [`WindowOperationResult::Performed`] with [`SupportLevel::Limited`]:
    /// the operation was satisfied conditionally, with documented restrictions
    /// (the spec §6.2 *conditional* category).
    #[must_use]
    pub const fn conditional() -> Self {
        Self::Performed(SupportLevel::Limited)
    }

    /// A [`WindowOperationResult::Failed`] carrying the given reason (spec §6.2).
    #[must_use]
    pub fn failed(reason: impl Into<Box<str>>) -> Self {
        Self::Failed(reason.into())
    }

    /// Returns whether the operation was satisfied (any
    /// [`WindowOperationResult::Performed`] outcome).
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Performed(_))
    }

    /// Returns whether the operation is unsupported on this platform.
    #[must_use]
    pub const fn is_unsupported(&self) -> bool {
        matches!(self, Self::Unsupported)
    }

    /// Returns the support level of this outcome, if it maps to one.
    ///
    /// [`WindowOperationResult::Performed`] returns its recorded level;
    /// [`WindowOperationResult::Unsupported`] returns
    /// [`SupportLevel::Unavailable`]; a runtime
    /// [`WindowOperationResult::Failed`] returns `None`, as failure is not a
    /// support category.
    #[must_use]
    pub const fn support_level(&self) -> Option<SupportLevel> {
        match self {
            Self::Performed(level) => Some(*level),
            Self::Unsupported => Some(SupportLevel::Unavailable),
            Self::Failed(_) => None,
        }
    }

    /// Returns the failure reason for a [`WindowOperationResult::Failed`]
    /// outcome, else `None`.
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
