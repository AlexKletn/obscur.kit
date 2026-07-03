//! Current state of a permission grant.

/// The state of a permission for a capability (spec §8.57).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionState {
    /// Granted for use.
    Granted,
    /// Explicitly denied.
    Denied,
    /// Not yet decided; a request would prompt.
    Prompt,
    /// The underlying capability is unsupported here.
    Unsupported,
}
