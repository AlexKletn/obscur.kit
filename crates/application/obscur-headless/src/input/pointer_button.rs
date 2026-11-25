//! Pointer button identity for synthetic pointer input (spec §7.4).

#[doc = include_str!("../../docs/api/input/pointer_button.doc/PointerButton.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PointerButton {
    /// The primary button (typically the left mouse button). This is the default.
    #[default]
    Primary,
    /// The secondary button (typically the right mouse button).
    Secondary,
    /// The middle button (typically the mouse wheel button).
    Middle,
}

#[cfg(test)]
mod tests {
    use super::PointerButton;

    #[test]
    fn default_is_primary() {
        assert_eq!(PointerButton::default(), PointerButton::Primary);
    }

    #[test]
    fn variants_are_distinct() {
        assert_ne!(PointerButton::Primary, PointerButton::Secondary);
        assert_ne!(PointerButton::Secondary, PointerButton::Middle);
        assert_ne!(PointerButton::Primary, PointerButton::Middle);
    }

    #[test]
    fn is_copy_and_eq() {
        let button = PointerButton::Secondary;
        let copied = button;
        assert_eq!(button, copied);
    }
}
