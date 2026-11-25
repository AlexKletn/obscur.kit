//! The logical identity of plugin-owned state in one installation.

use obscur_plugin_model::{
    identity::{PluginId, PluginPrincipal},
    state::StateScope,
};

/// The logical identity of plugin-owned state in one installation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StorageNamespace {
    principal: PluginPrincipal,
    scope: StateScope,
}

impl StorageNamespace {
    /// Identifies global state within a plugin installation.
    #[must_use]
    pub fn new(plugin: PluginId, installation: impl Into<Box<str>>) -> Self {
        Self {
            principal: PluginPrincipal::new(plugin, installation),
            scope: StateScope::Global,
        }
    }

    /// Selects a state area for this installation.
    #[must_use]
    pub fn with_scope(mut self, scope: StateScope) -> Self {
        self.scope = scope;
        self
    }

    /// Returns the selected state area.
    #[must_use]
    pub const fn scope(&self) -> &StateScope {
        &self.scope
    }

    /// Returns the plugin that owns this state.
    #[must_use]
    pub const fn plugin(&self) -> &PluginId {
        self.principal.plugin()
    }

    /// Returns the installation identity assigned by the host.
    #[must_use]
    pub fn installation(&self) -> &str {
        self.principal.installation()
    }
}
