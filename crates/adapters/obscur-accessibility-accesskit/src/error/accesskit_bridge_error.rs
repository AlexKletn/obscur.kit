//! The AccessKit accessibility adapter's own error type (spec §7.1).

/// A failure raised while converting the domain semantic tree into AccessKit
/// types (spec §7.1, §2.7).
///
/// This is the adapter's own diagnostic, surfaced instead of a panic when a
/// conversion cannot faithfully produce an [`accesskit::TreeUpdate`]. The
/// determinism boundary of spec §7.1 requires the tree structure, roles, and
/// states to map faithfully; when that is impossible the adapter reports the
/// concrete cause here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessKitBridgeError {
    /// The semantic snapshot has no root node, so AccessKit — which requires a
    /// single root for every [`accesskit::Tree`] — cannot be built from it.
    MissingRoot,
    /// A semantic role had no AccessKit equivalent and could not be mapped. The
    /// payload names the offending role for diagnostics. The built-in
    /// [`RoleConverter`](crate::conversion::RoleConverter) never produces this
    /// (it always falls back to [`accesskit::Role::Unknown`]); it exists for
    /// stricter callers that reject approximations.
    UnmappableRole(Box<str>),
    /// A generic AccessKit backend failure, carrying a human-readable reason.
    Backend(Box<str>),
}

impl std::fmt::Display for AccessKitBridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingRoot => {
                f.write_str("semantic snapshot has no root; cannot build an AccessKit tree")
            }
            Self::UnmappableRole(role) => {
                write!(f, "semantic role `{role}` has no AccessKit equivalent")
            }
            Self::Backend(reason) => write!(f, "accesskit backend failure: {reason}"),
        }
    }
}

impl std::error::Error for AccessKitBridgeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_renders_each_variant() {
        assert_eq!(
            AccessKitBridgeError::MissingRoot.to_string(),
            "semantic snapshot has no root; cannot build an AccessKit tree"
        );
        assert_eq!(
            AccessKitBridgeError::UnmappableRole("Widget".into()).to_string(),
            "semantic role `Widget` has no AccessKit equivalent"
        );
        assert_eq!(
            AccessKitBridgeError::Backend("boom".into()).to_string(),
            "accesskit backend failure: boom"
        );
    }

    #[test]
    fn is_std_error() {
        fn assert_error<E: std::error::Error>(_: &E) {}
        assert_error(&AccessKitBridgeError::MissingRoot);
    }

    #[test]
    fn equality_holds() {
        assert_eq!(
            AccessKitBridgeError::MissingRoot,
            AccessKitBridgeError::MissingRoot
        );
        assert_ne!(
            AccessKitBridgeError::MissingRoot,
            AccessKitBridgeError::Backend("x".into())
        );
    }
}
