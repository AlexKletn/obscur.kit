//! Diagnostic information for a plugin execution failure.

use obscur_plugin_api::status::PluginApiError;
use obscur_plugin_model::manifest::Version;

/// The most recent execution failure retained by the manager for a plugin ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginFailure {
    /// The version of the failing instance, if registered.
    pub version: Option<Version>,
    /// The original execution failure.
    pub error: PluginApiError,
}
