//! The port a platform implements to read its desktop's appearance.

use obscur_capability::SupportLevel;

use super::{AppearanceCallback, AppearanceWatch, SystemAppearance};

#[doc = include_str!("../../docs/api/appearance/appearance_provider.doc/AppearanceProvider.md")]
pub trait AppearanceProvider {
    #[doc = include_str!("../../docs/api/appearance/appearance_provider.doc/appearance.md")]
    fn appearance(&self) -> SystemAppearance;

    #[doc = include_str!("../../docs/api/appearance/appearance_provider.doc/support.md")]
    fn support(&self) -> SupportLevel;

    /// Whether asking this source is worth doing at all.
    fn answers(&self) -> bool {
        !matches!(self.support(), SupportLevel::Unavailable)
    }

    #[doc = include_str!("../../docs/api/appearance/appearance_provider.doc/watch_support.md")]
    fn watch_support(&self) -> SupportLevel {
        SupportLevel::Unavailable
    }

    #[doc = include_str!("../../docs/api/appearance/appearance_provider.doc/watch.md")]
    fn watch(&self, on_change: AppearanceCallback) -> Option<AppearanceWatch> {
        let _ = on_change;
        None
    }

    /// Whether asking to be told is worth doing at all.
    fn notifies(&self) -> bool {
        !matches!(self.watch_support(), SupportLevel::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::super::NullAppearanceProvider;
    use super::*;

    #[test]
    fn the_port_is_object_safe() {
        fn assert_obj_safe(_: &dyn AppearanceProvider) {}
        assert_obj_safe(&NullAppearanceProvider::new());
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
