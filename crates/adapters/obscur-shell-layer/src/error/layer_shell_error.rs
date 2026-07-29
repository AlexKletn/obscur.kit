//! Failures opening or driving a layer surface.

use core::fmt;

/// What can go wrong between asking for a layer surface and having one.
///
/// Every variant carries the compositor's or the library's own message rather
/// than a rewritten one: when a compositor refuses a protocol, its reason is the
/// useful half.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerShellError {
    /// No Wayland display to connect to — not a Wayland session at all.
    NoDisplay(String),
    /// Connected, but the compositor does not implement `wlr-layer-shell`.
    ///
    /// This is the expected outcome on GNOME (Mutter), which has declined the
    /// protocol, and on any non-wlroots compositor that has not adopted it. It is
    /// a capability answer, not a bug: a host should fall back to an ordinary
    /// window rather than treat it as a failure.
    Unsupported,
    /// A global the shell needs (`wl_compositor`, `wl_seat`) is missing.
    MissingGlobal(String),
    /// The connection failed while dispatching.
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
