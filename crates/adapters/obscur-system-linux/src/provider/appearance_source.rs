//! Which desktop setting store to read appearance from.

#[doc = include_str!("../../docs/api/provider/appearance_source.doc/AppearanceSource.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppearanceSource {
    #[doc = include_str!("../../docs/api/provider/appearance_source.doc/Auto.md")]
    #[default]
    Auto,
    #[doc = include_str!("../../docs/api/provider/appearance_source.doc/Portal.md")]
    Portal,
    #[doc = include_str!("../../docs/api/provider/appearance_source.doc/Gtk.md")]
    Gtk,
    #[doc = include_str!("../../docs/api/provider/appearance_source.doc/Qt.md")]
    Qt,
}

impl AppearanceSource {
    #[doc = include_str!("../../docs/api/provider/appearance_source.doc/order.md")]
    #[must_use]
    pub const fn order(self) -> &'static [Self] {
        match self {
            // Portal first because it is the answer both toolkits themselves
            // read and the only one that survives a sandbox; then GTK, which
            // states fonts and interface settings the portal has no key for;
            // then Qt, which is the only source on this list that states a
            // palette at all.
            Self::Auto => &[Self::Portal, Self::Gtk, Self::Qt],
            Self::Portal => &[Self::Portal],
            Self::Gtk => &[Self::Gtk],
            Self::Qt => &[Self::Qt],
        }
    }

    pub(super) fn session_order(self, desktop: &str) -> &'static [Self] {
        if !self.is_auto() {
            return self.order();
        }
        for name in desktop.split(':').map(str::trim) {
            if name.eq_ignore_ascii_case("KDE") || name.eq_ignore_ascii_case("Plasma") {
                return &[Self::Portal, Self::Qt];
            }
            if name.eq_ignore_ascii_case("GNOME") {
                return &[Self::Portal, Self::Gtk];
            }
        }
        &[Self::Portal]
    }

    /// Whether this asks everything rather than one named store.
    #[must_use]
    pub const fn is_auto(self) -> bool {
        matches!(self, Self::Auto)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_asks_everything() {
        assert!(AppearanceSource::default().is_auto());
        assert_eq!(AppearanceSource::Auto.order().len(), 3);
    }

    #[test]
    fn a_named_source_asks_only_itself() {
        // The integrator's choice is a choice, not a preference: naming Qt on a
        // GNOME desktop must answer what Qt says, not quietly fall through to
        // `gsettings` and look like it worked.
        for source in [
            AppearanceSource::Portal,
            AppearanceSource::Gtk,
            AppearanceSource::Qt,
        ] {
            assert_eq!(source.order(), &[source]);
            assert!(!source.is_auto());
        }
    }

    #[test]
    fn auto_asks_the_portal_before_either_toolkit() {
        // Order is the whole content of `Auto`, and the portal leads because it
        // is the only source that answers inside a sandbox.
        assert_eq!(AppearanceSource::Auto.order()[0], AppearanceSource::Portal);
    }

    #[test]
    fn auto_asks_qt_last_and_that_is_deliberate() {
        // Last, but never dropped: it is the only source of a palette, so an
        // `Auto` that stopped at the first answering source would lose the
        // colours on a KDE desktop that also runs a portal.
        let order = AppearanceSource::Auto.order();
        assert_eq!(order[order.len() - 1], AppearanceSource::Qt);
    }
}

#[cfg(test)]
#[path = "appearance_source.tests.rs"]
mod session_tests;
