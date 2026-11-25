//! How long a permission grant lasts.

/// The lifetime of a permission grant (spec §8.57).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionDuration {
    /// Valid for a single use.
    OneShot,
    /// Valid for the current session.
    Session,
    /// Persisted across sessions until revoked.
    Persistent,
}
