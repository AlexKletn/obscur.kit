//! `PluginApiError` (skeleton stub for the spec-defined entity; not yet implemented).

/// A failure reported while activating, deactivating, or rendering a built-in plugin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginApiError {
    /// An application-defined plugin failure.
    Rejected(Box<str>),
    /// The host already has an active plugin with this ID.
    AlreadyActive,
    /// The plugin has no active instance.
    NotActive,
    /// The host has not published the extension point.
    ExtensionUnavailable,
    /// The plugin staged the same contribution ID more than once.
    DuplicateContribution(Box<str>),
    /// The settings route conflicts with an existing host route.
    RouteConflict(Box<str>),
    /// Trusted plugin code panicked while the host invoked it.
    Panicked,
    /// The plugin executor faulted during guest execution.
    ExecutorCrashed(Box<str>),
    /// A resource limit stopped guest execution.
    ExecutionQuotaExceeded(Box<str>),
    /// The remaining backoff before the host can retry activation.
    RestartDeferred(std::time::Duration),
    /// The plugin reached its failure limit and requires an explicit reset.
    RestartBlocked,
    /// Guest execution reached the host deadline.
    ExecutionTimedOut,
}

impl std::fmt::Display for PluginApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rejected(message) => f.write_str(message),
            Self::AlreadyActive => f.write_str("plugin is already active"),
            Self::NotActive => f.write_str("plugin is not active"),
            Self::ExtensionUnavailable => {
                f.write_str("integration point is not published by the host")
            }
            Self::DuplicateContribution(id) => write!(f, "duplicate plugin contribution {id}"),
            Self::RouteConflict(route) => write!(f, "plugin route conflicts with {route}"),
            Self::Panicked => f.write_str("plugin panicked"),
            Self::ExecutorCrashed(reason) => write!(f, "plugin executor crashed: {reason}"),
            Self::ExecutionQuotaExceeded(reason) => {
                write!(f, "plugin exceeded its execution quota: {reason}")
            }
            Self::RestartDeferred(delay) => write!(f, "plugin restart deferred for {delay:?}"),
            Self::RestartBlocked => f.write_str("plugin restart requires a host reset"),
            Self::ExecutionTimedOut => f.write_str("plugin execution timed out"),
        }
    }
}

impl std::error::Error for PluginApiError {}
