#![doc = include_str!("accesskit_bridge_error.doc/_module.md")]

#[doc = include_str!("accesskit_bridge_error.doc/AccessKitBridgeError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessKitBridgeError {
    #[doc = include_str!("accesskit_bridge_error.doc/MissingRoot.md")]
    MissingRoot,
    #[doc = include_str!("accesskit_bridge_error.doc/UnmappableRole.md")]
    UnmappableRole(Box<str>),
    #[doc = include_str!("accesskit_bridge_error.doc/Backend.md")]
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
