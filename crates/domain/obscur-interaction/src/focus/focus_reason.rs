#[doc = include_str!("../../docs/api/focus/focus_request.doc/FocusReason.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusReason {
    /// Focus moved by explicit application code.
    Programmatic,
    /// Focus moved as a result of a pointer interaction.
    Pointer,
    /// Focus moved by keyboard navigation (e.g. Tab).
    Keyboard,
    /// Focus moved by restoring a previously focused node when a scope closed.
    Restore,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_reason_variants_are_distinct() {
        assert_ne!(FocusReason::Programmatic, FocusReason::Keyboard);
        assert_ne!(FocusReason::Pointer, FocusReason::Restore);
    }
}
