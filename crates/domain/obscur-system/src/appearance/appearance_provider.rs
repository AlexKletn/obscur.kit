#![doc = include_str!("appearance_provider.doc/_module.md")]

use std::sync::Arc;

use obscur_capability::SupportLevel;

use super::{AppearanceWatch, SystemAppearance};

#[doc = include_str!("appearance_provider.doc/AppearanceCallback.md")]
pub type AppearanceCallback = Arc<dyn Fn() + Send + Sync + 'static>;

#[doc = include_str!("appearance_provider.doc/AppearanceProvider.md")]
pub trait AppearanceProvider {
    #[doc = include_str!("appearance_provider.doc/appearance.md")]
    fn appearance(&self) -> SystemAppearance;

    #[doc = include_str!("appearance_provider.doc/support.md")]
    fn support(&self) -> SupportLevel;

    #[doc = include_str!("appearance_provider.doc/answers.md")]
    fn answers(&self) -> bool {
        !matches!(self.support(), SupportLevel::Unavailable)
    }

    #[doc = include_str!("appearance_provider.doc/watch_support.md")]
    fn watch_support(&self) -> SupportLevel {
        SupportLevel::Unavailable
    }

    #[doc = include_str!("appearance_provider.doc/watch.md")]
    fn watch(&self, on_change: AppearanceCallback) -> Option<AppearanceWatch> {
        let _ = on_change;
        None
    }

    #[doc = include_str!("appearance_provider.doc/notifies.md")]
    fn notifies(&self) -> bool {
        !matches!(self.watch_support(), SupportLevel::Unavailable)
    }
}

#[doc = include_str!("appearance_provider.doc/NullAppearanceProvider.md")]
#[derive(Debug, Default)]
pub struct NullAppearanceProvider;

impl NullAppearanceProvider {
    #[doc = include_str!("appearance_provider.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl AppearanceProvider for NullAppearanceProvider {
    fn appearance(&self) -> SystemAppearance {
        SystemAppearance::default()
    }

    fn support(&self) -> SupportLevel {
        SupportLevel::Unavailable
    }

    fn watch_support(&self) -> SupportLevel {
        SupportLevel::Unavailable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_port_is_object_safe() {
        fn assert_obj_safe(_: &dyn AppearanceProvider) {}
        assert_obj_safe(&NullAppearanceProvider::new());
    }

    #[test]
    fn a_platform_with_no_desktop_settings_answers_nothing_and_says_so() {
        let provider = NullAppearanceProvider::new();
        assert!(provider.appearance().is_empty());
        assert_eq!(provider.support(), SupportLevel::Unavailable);
        assert!(!provider.answers());
    }

    #[test]
    fn a_source_says_nothing_about_changes_unless_it_can() {
        // The default, and the honest one: most settings stores have no signal,
        // so a provider that does nothing inherits "you will not be told".
        let provider = NullAppearanceProvider::new();
        assert_eq!(provider.watch_support(), SupportLevel::Unavailable);
        assert!(!provider.notifies());
        assert!(provider.watch(Arc::new(|| {})).is_none());
    }

    #[test]
    fn reading_and_being_told_are_two_separate_capabilities() {
        // A source can answer everything and notify nothing — `kdeglobals` is
        // exactly that — so one level cannot stand for the other.
        struct Complete;
        impl AppearanceProvider for Complete {
            fn appearance(&self) -> SystemAppearance {
                SystemAppearance::default()
            }
            fn support(&self) -> SupportLevel {
                SupportLevel::Native
            }
        }
        assert_eq!(Complete.support(), SupportLevel::Native);
        assert_eq!(Complete.watch_support(), SupportLevel::Unavailable);
        assert!(Complete.answers());
        assert!(!Complete.notifies());
    }

    #[test]
    fn a_provider_that_answers_partially_still_answers() {
        // `Limited` is the honest level for a source that knows the scheme and
        // not the palette, and a host must not read it as "nothing here".
        struct Partial;
        impl AppearanceProvider for Partial {
            fn appearance(&self) -> SystemAppearance {
                SystemAppearance::default()
            }
            fn support(&self) -> SupportLevel {
                SupportLevel::Limited
            }
        }
        assert!(Partial.answers());
    }
}
