#[doc = include_str!("../../docs/api/request/activate_window.doc/ActivationResult.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivationResult {
    /// The window was activated (and any requested element focused).
    Activated,
    #[doc = include_str!("../../docs/api/request/activate_window.doc/Denied.md")]
    Denied,
    /// The platform does not support programmatic activation/focus.
    Unsupported,
    /// Activation was attempted but failed at runtime, carrying a reason.
    Failed(Box<str>),
}

impl ActivationResult {
    /// Returns whether the request resulted in activation.
    #[must_use]
    pub const fn is_activated(&self) -> bool {
        matches!(self, Self::Activated)
    }
}

#[cfg(test)]
mod tests {
    use super::ActivationResult;

    #[test]
    fn activation_result_predicate() {
        assert!(ActivationResult::Activated.is_activated());
        assert!(!ActivationResult::Denied.is_activated());
        assert!(!ActivationResult::Unsupported.is_activated());
        assert!(!ActivationResult::Failed(Box::from("x")).is_activated());
    }
}
