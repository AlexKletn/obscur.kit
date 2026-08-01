#![doc = include_str!("layer_keyboard_interactivity.doc/_module.md")]

#[doc = include_str!("layer_keyboard_interactivity.doc/LayerKeyboardInteractivity.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LayerKeyboardInteractivity {
    #[doc = include_str!("layer_keyboard_interactivity.doc/None.md")]
    #[default]
    None,
    #[doc = include_str!("layer_keyboard_interactivity.doc/Exclusive.md")]
    Exclusive,
    #[doc = include_str!("layer_keyboard_interactivity.doc/OnDemand.md")]
    OnDemand,
}

impl LayerKeyboardInteractivity {
    #[doc = include_str!("layer_keyboard_interactivity.doc/takes_keys.md")]
    #[must_use]
    pub const fn takes_keys(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_never_takes_the_keyboard() {
        assert_eq!(
            LayerKeyboardInteractivity::default(),
            LayerKeyboardInteractivity::None
        );
        assert!(!LayerKeyboardInteractivity::default().takes_keys());
    }

    #[test]
    fn both_interactive_settings_can_route_keys() {
        assert!(LayerKeyboardInteractivity::Exclusive.takes_keys());
        assert!(LayerKeyboardInteractivity::OnDemand.takes_keys());
    }
}
