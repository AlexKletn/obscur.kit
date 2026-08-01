#![doc = include_str!("layer_shell_error.doc/_module.md")]

use core::fmt;

#[doc = include_str!("layer_shell_error.doc/LayerShellError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerShellError {
    #[doc = include_str!("layer_shell_error.doc/NoDisplay.md")]
    NoDisplay(String),
    #[doc = include_str!("layer_shell_error.doc/Unsupported.md")]
    Unsupported,
    #[doc = include_str!("layer_shell_error.doc/MissingGlobal.md")]
    MissingGlobal(String),
    #[doc = include_str!("layer_shell_error.doc/Dispatch.md")]
    Dispatch(String),
}

impl fmt::Display for LayerShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDisplay(why) => write!(f, "no wayland display: {why}"),
            Self::Unsupported => {
                write!(f, "the compositor does not implement wlr-layer-shell")
            }
            Self::MissingGlobal(name) => write!(f, "wayland global unavailable: {name}"),
            Self::Dispatch(why) => write!(f, "wayland dispatch failed: {why}"),
        }
    }
}

impl std::error::Error for LayerShellError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// The one variant a host is expected to branch on says what it means
    /// without a cause string, because there is no cause: the compositor simply
    /// does not offer the protocol.
    #[test]
    fn the_unsupported_case_reads_as_a_capability_answer() {
        assert_eq!(
            LayerShellError::Unsupported.to_string(),
            "the compositor does not implement wlr-layer-shell"
        );
    }

    #[test]
    fn every_other_variant_keeps_the_underlying_message() {
        assert!(
            LayerShellError::NoDisplay("XDG_RUNTIME_DIR unset".into())
                .to_string()
                .contains("XDG_RUNTIME_DIR unset")
        );
        assert!(
            LayerShellError::Dispatch("broken pipe".into())
                .to_string()
                .contains("broken pipe")
        );
    }
}
