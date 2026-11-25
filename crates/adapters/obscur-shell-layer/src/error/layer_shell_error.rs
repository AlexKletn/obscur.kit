//! Failures opening or driving a layer surface.

use core::fmt;
use wayland_client::{ConnectError, DispatchError, backend::WaylandError, globals::GlobalError};

#[doc = include_str!("../../docs/api/error/layer_shell_error.doc/LayerShellError.md")]
#[derive(Debug)]
pub enum LayerShellError {
    /// No Wayland display to connect to — not a Wayland session at all.
    NoDisplay(ConnectError),
    /// The compositor registry could not be initialized.
    Registry(GlobalError),
    /// The compositor failed to complete a synchronous request.
    Roundtrip(DispatchError),
    /// The event loop could not be created or driven.
    EventLoop(calloop::Error),
    /// The event loop could not register a source.
    RegisterSource(calloop::Error),
    /// The OS could not allocate a wake source.
    WakeSource(std::io::Error),
    /// Pending requests could not be sent to the compositor.
    Flush(WaylandError),
    #[doc = include_str!("../../docs/api/error/layer_shell_error.doc/Unsupported.md")]
    Unsupported,
    /// A global the shell needs (`wl_compositor`, `wl_seat`) is missing.
    MissingGlobal(String),
    /// The requested layer settings cannot be applied.
    InvalidSettings(String),
    /// The connection failed while dispatching.
    Dispatch(String),
}

impl fmt::Display for LayerShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDisplay(why) => write!(f, "no wayland display: {why}"),
            Self::Registry(error) => write!(f, "wayland registry initialization failed: {error}"),
            Self::Roundtrip(error) => write!(f, "wayland roundtrip failed: {error}"),
            Self::EventLoop(error) => write!(f, "wayland event loop failed: {error}"),
            Self::RegisterSource(error) => write!(f, "wayland source registration failed: {error}"),
            Self::WakeSource(error) => write!(f, "could not create a layer wake source: {error}"),
            Self::Flush(error) => write!(f, "wayland flush failed: {error}"),
            Self::Unsupported => {
                write!(f, "the compositor does not implement wlr-layer-shell")
            }
            Self::MissingGlobal(name) => write!(f, "wayland global unavailable: {name}"),
            Self::InvalidSettings(why) => write!(f, "invalid layer settings: {why}"),
            Self::Dispatch(why) => write!(f, "wayland dispatch failed: {why}"),
        }
    }
}

impl std::error::Error for LayerShellError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NoDisplay(error) => Some(error),
            Self::Registry(error) => Some(error),
            Self::Roundtrip(error) => Some(error),
            Self::EventLoop(error) | Self::RegisterSource(error) => Some(error),
            Self::WakeSource(error) => Some(error),
            Self::Flush(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn a_failed_connection_exposes_the_wayland_error() {
        let error = LayerShellError::NoDisplay(ConnectError::InvalidFd);
        let source = error
            .source()
            .and_then(|source| source.downcast_ref::<ConnectError>());

        assert!(matches!(source, Some(ConnectError::InvalidFd)));
    }

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
            LayerShellError::NoDisplay(ConnectError::InvalidFd)
                .to_string()
                .contains("WAYLAND_SOCKET")
        );
        assert!(
            LayerShellError::Dispatch("broken pipe".into())
                .to_string()
                .contains("broken pipe")
        );
    }
}
