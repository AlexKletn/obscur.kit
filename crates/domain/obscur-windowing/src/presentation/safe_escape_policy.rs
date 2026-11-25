#[doc = include_str!("../../docs/api/presentation/presentation_mode.doc/SafeEscapePolicy.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SafeEscapePolicy {
    /// The user may leave at any time with the conventional gesture.
    Allow,
    /// The user must hold the escape gesture to leave, guarding against
    /// accidental exits.
    RequireHold,
    /// The application-provided exit is the only way out.
    Disabled,
}

impl Default for SafeEscapePolicy {
    /// The default policy is [`SafeEscapePolicy::Allow`], the safe default.
    fn default() -> Self {
        Self::Allow
    }
}
