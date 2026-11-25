//! How much of the keyboard a layer surface asks for.

#[doc = include_str!("../../docs/api/layer/layer_focus_policy.doc/LayerFocusPolicy.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LayerFocusPolicy {
    #[doc = include_str!("../../docs/api/layer/layer_focus_policy.doc/Never.md")]
    #[default]
    Never,
    #[doc = include_str!("../../docs/api/layer/layer_focus_policy.doc/OnDemand.md")]
    OnDemand,
    #[doc = include_str!("../../docs/api/layer/layer_focus_policy.doc/Exclusive.md")]
    Exclusive,
}

impl LayerFocusPolicy {
    /// Whether this policy routes any key to the surface at all.
    #[must_use]
    pub const fn takes_keys(self) -> bool {
        !matches!(self, Self::Never)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_never_takes_the_keyboard() {
        assert_eq!(LayerFocusPolicy::default(), LayerFocusPolicy::Never);
        assert!(!LayerFocusPolicy::default().takes_keys());
    }

    #[test]
    fn both_interactive_settings_can_route_keys() {
        assert!(LayerFocusPolicy::OnDemand.takes_keys());
        assert!(LayerFocusPolicy::Exclusive.takes_keys());
    }
}
