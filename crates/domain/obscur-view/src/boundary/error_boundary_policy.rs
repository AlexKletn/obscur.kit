#![doc = include_str!("error_boundary_policy.doc/_module.md")]

#[doc = include_str!("error_boundary_policy.doc/ErrorBoundaryPolicy.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorBoundaryPolicy {
    #[doc = include_str!("error_boundary_policy.doc/enabled.md")]
    pub enabled: bool,
    #[doc = include_str!("error_boundary_policy.doc/max_retries.md")]
    pub max_retries: u32,
}

impl ErrorBoundaryPolicy {
    #[doc = include_str!("error_boundary_policy.doc/DEFAULT_MAX_RETRIES.md")]
    pub const DEFAULT_MAX_RETRIES: u32 = 1;

    #[doc = include_str!("error_boundary_policy.doc/new.md")]
    #[must_use]
    pub const fn new(enabled: bool, max_retries: u32) -> Self {
        Self {
            enabled,
            max_retries,
        }
    }

    #[doc = include_str!("error_boundary_policy.doc/disabled.md")]
    #[must_use]
    pub const fn disabled() -> Self {
        Self::new(false, 0)
    }

    #[doc = include_str!("error_boundary_policy.doc/with_max_retries.md")]
    #[must_use]
    pub const fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    #[doc = include_str!("error_boundary_policy.doc/is_enabled.md")]
    #[must_use]
    pub const fn is_enabled(self) -> bool {
        self.enabled
    }
}

impl Default for ErrorBoundaryPolicy {
    #[doc = include_str!("error_boundary_policy.doc/default.md")]
    fn default() -> Self {
        Self::new(true, Self::DEFAULT_MAX_RETRIES)
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorBoundaryPolicy;

    #[test]
    fn default_is_enabled_with_budget() {
        let policy = ErrorBoundaryPolicy::default();
        assert!(policy.is_enabled());
        assert_eq!(policy.max_retries, ErrorBoundaryPolicy::DEFAULT_MAX_RETRIES);
    }

    #[test]
    fn disabled_has_no_budget() {
        let policy = ErrorBoundaryPolicy::disabled();
        assert!(!policy.is_enabled());
        assert_eq!(policy.max_retries, 0);
    }

    #[test]
    fn with_max_retries_overrides_budget() {
        let policy = ErrorBoundaryPolicy::default().with_max_retries(5);
        assert!(policy.is_enabled());
        assert_eq!(policy.max_retries, 5);
    }
}
