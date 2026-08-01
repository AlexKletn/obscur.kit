#![doc = include_str!("pointer_button.doc/_module.md")]

#[doc = include_str!("pointer_button.doc/PointerButton.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PointerButton {
    #[doc = include_str!("pointer_button.doc/Primary.md")]
    #[default]
    Primary,
    #[doc = include_str!("pointer_button.doc/Secondary.md")]
    Secondary,
    #[doc = include_str!("pointer_button.doc/Middle.md")]
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
