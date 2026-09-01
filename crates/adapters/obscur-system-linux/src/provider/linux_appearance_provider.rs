#![doc = include_str!("linux_appearance_provider.doc/_module.md")]

use std::sync::Arc;

use obscur_capability::SupportLevel;
use obscur_system::appearance::{
    AppearanceCallback, AppearanceProvider, AppearanceWatch, SystemAppearance,
};

use super::appearance_source::AppearanceSource;
use super::gtk_appearance_provider::GtkAppearanceProvider;
use super::portal_appearance_provider::PortalAppearanceProvider;
use super::qt_appearance_provider::QtAppearanceProvider;

#[doc = include_str!("linux_appearance_provider.doc/LinuxAppearanceProvider.md")]
#[derive(Debug, Clone, Default)]
pub struct LinuxAppearanceProvider {
    source: AppearanceSource,
    portal: PortalAppearanceProvider,
    gtk: GtkAppearanceProvider,
    qt: QtAppearanceProvider,
}

impl LinuxAppearanceProvider {
    #[doc = include_str!("linux_appearance_provider.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::from_source(AppearanceSource::Auto)
    }

    #[doc = include_str!("linux_appearance_provider.doc/from_source.md")]
    #[must_use]
    pub fn from_source(source: AppearanceSource) -> Self {
        Self {
            source,
            portal: PortalAppearanceProvider::new(),
            gtk: GtkAppearanceProvider::new(),
            qt: QtAppearanceProvider::new(),
        }
    }

    #[doc = include_str!("linux_appearance_provider.doc/source.md")]
    #[must_use]
    pub const fn source(&self) -> AppearanceSource {
        self.source
    }

    #[doc = include_str!("linux_appearance_provider.doc/with_portal.md")]
    #[must_use]
    pub fn with_portal(mut self, portal: PortalAppearanceProvider) -> Self {
        self.portal = portal;
        self
    }

    #[doc = include_str!("linux_appearance_provider.doc/with_gtk.md")]
    #[must_use]
    pub fn with_gtk(mut self, gtk: GtkAppearanceProvider) -> Self {
        self.gtk = gtk;
        self
    }

    #[doc = include_str!("linux_appearance_provider.doc/with_qt.md")]
    #[must_use]
    pub fn with_qt(mut self, qt: QtAppearanceProvider) -> Self {
        self.qt = qt;
        self
    }

    #[doc = include_str!("linux_appearance_provider.doc/provider_for.md")]
    fn provider_for(&self, source: AppearanceSource) -> Option<&dyn AppearanceProvider> {
        match source {
            AppearanceSource::Portal => Some(&self.portal),
            AppearanceSource::Gtk => Some(&self.gtk),
            AppearanceSource::Qt => Some(&self.qt),
            AppearanceSource::Auto => None,
        }
    }
}

impl AppearanceProvider for LinuxAppearanceProvider {
    fn appearance(&self) -> SystemAppearance {
        // Merged rather than first-wins. A KDE desktop running a portal answers
        // the scheme twice and the palette once, and stopping at the first
        // source that said anything would drop every colour on the machine.
        // Earlier sources outrank later ones field by field.
        self.source
            .order()
            .iter()
            .filter_map(|source| self.provider_for(*source))
            .filter(|provider| provider.answers())
            .map(AppearanceProvider::appearance)
            .fold(SystemAppearance::default(), |merged, next| {
                merged.merge_under(&next)
            })
    }

    fn support(&self) -> SupportLevel {
        let levels: Vec<SupportLevel> = self
            .source
            .order()
            .iter()
            .filter_map(|source| self.provider_for(*source))
            .map(AppearanceProvider::support)
            .filter(|level| !matches!(level, SupportLevel::Unavailable))
            .collect();

        if levels.is_empty() {
            return SupportLevel::Unavailable;
        }
        // `Native` only where a source that states a whole palette answered:
        // the composite is exactly as complete as its most complete member, and
        // claiming more would put the caller back to guessing.
        if levels.contains(&SupportLevel::Native) {
            SupportLevel::Native
        } else {
            SupportLevel::Limited
        }
    }

    #[doc = include_str!("linux_appearance_provider.doc/watch_support.md")]
    fn watch_support(&self) -> SupportLevel {
        let asked: Vec<&dyn AppearanceProvider> = self
            .source
            .order()
            .iter()
            .filter_map(|source| self.provider_for(*source))
            .filter(|provider| provider.answers())
            .collect();

        if asked.is_empty() || !asked.iter().any(|p| p.notifies()) {
            return SupportLevel::Unavailable;
        }
        // `Limited` unless EVERY source being read can also report a change.
        // The case this exists for is a KDE desktop running a portal: the scheme
        // will be reported and the palette will not, so a host told `Native`
        // would stop polling and keep painting yesterday's colours.
        if asked.iter().all(|p| p.notifies()) {
            SupportLevel::Native
        } else {
            SupportLevel::Limited
        }
    }

    #[doc = include_str!("linux_appearance_provider.doc/watch.md")]
    fn watch(&self, on_change: AppearanceCallback) -> Option<AppearanceWatch> {
        let watches: Vec<AppearanceWatch> = self
            .source
            .order()
            .iter()
            .filter_map(|source| self.provider_for(*source))
            .filter(|provider| provider.answers())
            .filter_map(|provider| provider.watch(Arc::clone(&on_change)))
            .collect();

        if watches.is_empty() {
            return None;
        }
        // One guard over however many were started, so a caller holds one thing
        // and dropping it stops all of them.
        Some(AppearanceWatch::new(move || drop(watches)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn absent() -> LinuxAppearanceProvider {
        // Every source pointed at something that is not there, so the test says
        // the same thing on a KDE laptop and in a container.
        LinuxAppearanceProvider::new()
            .with_portal(PortalAppearanceProvider::with_tool("obscur-no-such-tool"))
            .with_gtk(GtkAppearanceProvider::with_tool("obscur-no-such-tool"))
            .with_qt(QtAppearanceProvider::at("/nonexistent/kdeglobals"))
    }

    #[test]
    fn the_default_asks_every_source() {
        assert_eq!(
            LinuxAppearanceProvider::new().source(),
            AppearanceSource::Auto
        );
    }

    #[test]
    fn a_desktop_with_nothing_to_read_answers_nothing_and_says_so() {
        let provider = absent();
        assert_eq!(provider.support(), SupportLevel::Unavailable);
        assert!(!provider.answers());
        assert!(provider.appearance().is_empty());
    }

    #[test]
    fn a_named_source_does_not_fall_through_to_the_others() {
        // The integrator chose Qt; a missing `kdeglobals` must answer nothing,
        // not quietly produce the GNOME session's scheme and look like it read
        // the file.
        let provider = LinuxAppearanceProvider::from_source(AppearanceSource::Qt)
            .with_qt(QtAppearanceProvider::at("/nonexistent/kdeglobals"));
        assert_eq!(provider.support(), SupportLevel::Unavailable);
        assert!(provider.appearance().is_empty());
    }

    #[test]
    fn the_composite_is_as_complete_as_its_most_complete_member() {
        // Nothing available: `Unavailable`. The `Native`/`Limited` arms need a
        // real source to observe, and this asserts the floor rather than
        // pretending otherwise.
        assert_eq!(absent().support(), SupportLevel::Unavailable);
    }

    #[test]
    fn every_source_is_reachable_through_the_composite() {
        let provider = LinuxAppearanceProvider::new();
        for source in [
            AppearanceSource::Portal,
            AppearanceSource::Gtk,
            AppearanceSource::Qt,
        ] {
            assert!(
                provider.provider_for(source).is_some(),
                "{source:?} has no provider behind it"
            );
        }
        assert!(
            provider.provider_for(AppearanceSource::Auto).is_none(),
            "`Auto` is an order, not a source"
        );
    }

    #[test]
    fn nothing_to_read_means_nothing_to_be_told_about() {
        let provider = absent();
        assert_eq!(provider.watch_support(), SupportLevel::Unavailable);
        assert!(!provider.notifies());
        assert!(provider.watch(Arc::new(|| {})).is_none());
    }

    #[test]
    fn a_source_that_cannot_report_changes_holds_the_composite_to_limited() {
        // The KDE-with-a-portal case, stated as the rule rather than measured on
        // a desktop that may not be there: Qt is read and cannot notify, so the
        // composite must not claim it will report everything.
        let qt = QtAppearanceProvider::new();
        assert_eq!(
            qt.watch_support(),
            SupportLevel::Unavailable,
            "the file-backed source never notifies"
        );
        assert!(!qt.notifies());
    }

    #[test]
    fn the_port_is_object_safe_through_this_type() {
        fn assert_obj_safe(_: &dyn AppearanceProvider) {}
        assert_obj_safe(&LinuxAppearanceProvider::new());
    }
}
