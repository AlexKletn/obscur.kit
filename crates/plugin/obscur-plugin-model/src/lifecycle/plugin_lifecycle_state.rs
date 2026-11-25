//! `PluginLifecycleState` (skeleton stub for the spec-defined entity; not yet implemented).

/// The committed lifecycle states of a built-in plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginLifecycleState {
    /// The plugin completed activation and its contributions are available.
    Active,
    /// The plugin instance is retained and its contributions are withdrawn.
    Suspended,
    /// The host removed the plugin’s contributions and ended the active lifetime.
    Deactivated,
}
