//! Whether a layer surface takes keyboard focus (spec §6.5).

/// How much of the keyboard a layer surface asks for.
///
/// The default is [`None`](LayerKeyboardInteractivity::None) and that is the
/// right default: a panel that silently took keyboard focus would swallow every
/// shortcut on the desktop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LayerKeyboardInteractivity {
    /// The surface never receives keyboard input. Keys go to whatever window has
    /// focus, exactly as if the surface were not there.
    #[default]
    None,
    /// The surface takes keyboard focus for as long as it exists, and nothing
    /// else can have it. For lock screens; hostile in anything else.
    Exclusive,
    /// The surface takes focus when the user interacts with it, and gives it
    /// back. What an on-screen keyboard's host, a launcher or a search bar
    /// wants.
    OnDemand,
}

impl LayerKeyboardInteractivity {
    /// Whether this setting can ever route a key to the surface.
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
