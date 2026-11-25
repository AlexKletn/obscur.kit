use obscur_plugin_api::status::PluginApiError;
use obscur_plugin_model::manifest::{Version, VersionReq};

/// Why a candidate state could not be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateMigrationError {
    /// The source schema lies outside the declared range.
    IncompatibleSource {
        /// Schema version of the existing state.
        found: Version,
        /// Source range accepted by the migration.
        accepted: VersionReq,
    },
    /// The migration callback returned an error or panicked.
    Callback(PluginApiError),
}

impl std::fmt::Display for StateMigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IncompatibleSource { found, accepted } => {
                write!(
                    f,
                    "state schema {found} does not match source range {accepted}"
                )
            }
            Self::Callback(error) => write!(f, "state migration failed: {error}"),
        }
    }
}

impl std::error::Error for StateMigrationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IncompatibleSource { .. } => None,
            Self::Callback(error) => Some(error),
        }
    }
}
