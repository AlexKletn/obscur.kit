//! `ErrorBoundaryPolicy`: how a component's error boundary behaves.

/// The policy governing a component's error boundary (spec §1.7).
///
/// Every component is an error boundary by default, and a component may
/// customize or explicitly disable that boundary. This small value captures
/// both choices: whether the boundary is [`enabled`](Self::enabled) at all, and
/// how many times a contained failure may be retried before the fallback UI is
/// kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorBoundaryPolicy {
    /// Whether the boundary contains failures in this component's subtree.
    ///
    /// When `false` the component opts out and failures propagate to the
    /// nearest enclosing boundary instead.
    pub enabled: bool,
    /// How many times a contained failure may be retried before giving up and
    /// keeping the fallback UI.
    pub max_retries: u32,
}

impl ErrorBoundaryPolicy {
    /// The default retry budget for an enabled boundary.
    pub const DEFAULT_MAX_RETRIES: u32 = 1;

    /// Creates an explicit policy from its parts.
    #[must_use]
    pub const fn new(enabled: bool, max_retries: u32) -> Self {
        Self {
            enabled,
            max_retries,
        }
    }

    /// A disabled boundary: failures are not contained here and propagate
    /// outward. Its retry budget is zero.
    #[must_use]
    pub const fn disabled() -> Self {
        Self::new(false, 0)
    }

    /// Returns the policy with its retry budget set to `max_retries`.
    #[must_use]
    pub const fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Returns whether this boundary is enabled.
    #[must_use]
    pub const fn is_enabled(self) -> bool {
        self.enabled
    }
}

impl Default for ErrorBoundaryPolicy {
    /// The default boundary is enabled with a small retry budget, matching the
    /// spec's "every component is an error boundary by default".
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
