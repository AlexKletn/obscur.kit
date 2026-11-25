use obscur_capability::SupportLevel;

use super::LayerFeature;

/// Panel features reported by a concrete window-system adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerCapabilities {
    levels: [SupportLevel; LayerFeature::ALL.len()],
}

impl LayerCapabilities {
    /// Creates a capability snapshot by querying each panel feature.
    #[must_use]
    pub fn from_support(mut query: impl FnMut(LayerFeature) -> SupportLevel) -> Self {
        Self {
            levels: LayerFeature::ALL.map(&mut query),
        }
    }

    /// Reports the support level of the given feature.
    #[must_use]
    pub fn supports(&self, feature: LayerFeature) -> SupportLevel {
        self.levels[feature as usize]
    }
}
