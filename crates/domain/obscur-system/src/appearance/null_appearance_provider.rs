use obscur_capability::SupportLevel;

use super::{AppearanceProvider, SystemAppearance};

#[doc = include_str!("../../docs/api/appearance/appearance_provider.doc/NullAppearanceProvider.md")]
#[derive(Debug, Default)]
pub struct NullAppearanceProvider;

impl NullAppearanceProvider {
    /// A provider that knows nothing.
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
    use std::sync::Arc;

    use super::*;

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
}
