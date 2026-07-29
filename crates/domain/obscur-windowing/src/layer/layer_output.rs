//! Which output a layer surface is placed on (spec §6.5).

/// The output (monitor) a layer surface asks to live on.
///
/// # Why a name and not a handle
///
/// The domain has no Wayland objects in it and must not grow any, so a choice
/// of output has to be expressible as data. A connector name — `eDP-1`, `DP-2`,
/// `HDMI-A-1` — is what a user's configuration file says, what a compositor's
/// rules match on, and what survives being written to disk and read back, which
/// a protocol handle does not.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum LayerOutput {
    /// Let the compositor decide.
    ///
    /// In practice this is the output holding the pointer or the focus, which is
    /// the right default for a launcher or a notification and the wrong one for a
    /// panel that belongs to a particular screen.
    #[default]
    Compositor,
    /// A specific connector, by name.
    ///
    /// A name that matches no connected output is **not** an error: the surface
    /// falls back to the compositor's choice, because a monitor being unplugged
    /// must not stop the panel from appearing at all. A host that needs to know
    /// which it got asks the surface afterwards.
    Named(String),
}

impl LayerOutput {
    /// The connector name this asks for, if it names one.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Compositor => None,
            Self::Named(name) => Some(name),
        }
    }

    /// Whether the choice is the compositor's.
    #[must_use]
    pub const fn is_compositor_choice(&self) -> bool {
        matches!(self, Self::Compositor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_leaves_the_choice_to_the_compositor() {
        assert!(LayerOutput::default().is_compositor_choice());
        assert_eq!(LayerOutput::default().name(), None);
    }

    #[test]
    fn a_named_output_carries_its_connector() {
        let output = LayerOutput::Named("eDP-1".to_owned());
        assert_eq!(output.name(), Some("eDP-1"));
        assert!(!output.is_compositor_choice());
    }
}
