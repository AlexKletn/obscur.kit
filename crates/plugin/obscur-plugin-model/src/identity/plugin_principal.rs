//! Plugin security principals.

use super::PluginId;

/// Identity of a plugin installation across package updates.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginPrincipal {
    plugin: PluginId,
    installation: Box<str>,
}

impl PluginPrincipal {
    /// Identifies a plugin installation as assigned by the host.
    #[must_use]
    pub fn new(plugin: PluginId, installation: impl Into<Box<str>>) -> Self {
        Self {
            plugin,
            installation: installation.into(),
        }
    }

    /// Returns the plugin identifier.
    #[must_use]
    pub const fn plugin(&self) -> &PluginId {
        &self.plugin
    }

    /// Returns the host-assigned installation identity.
    #[must_use]
    pub fn installation(&self) -> &str {
        &self.installation
    }
}
