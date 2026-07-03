//! The [`WinitBackendError`] failure type for the winit window backend
//! (spec §6.1 windowing; §2.7 "Реализует `WindowBackend` и преобразует native
//! events в доменные события").

use obscur_windowing::port::WindowError;

/// A failure raised inside the winit window backend (spec §6.1).
///
/// This is the adapter-local error type for the platform boundary: it names the
/// concrete winit-side failures (event-loop and window construction, a missing
/// active event loop, and any other backend failure) while carrying only a
/// human-readable message so it stays `Clone`/`Eq` and free of borrowed winit
/// state. It is mapped into the domain's [`WindowError`] via
/// [`WinitBackendError::into_window_error`] before crossing back into the pure
/// windowing domain (§2.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WinitBackendError {
    /// The winit event loop could not be created; carries the reason.
    EventLoopCreation(Box<str>),
    /// A platform window could not be created; carries the reason.
    WindowCreation(Box<str>),
    /// An operation required an active event loop, but none was available.
    NoActiveEventLoop,
    /// Any other backend-side failure; carries the reason.
    Backend(Box<str>),
}

impl WinitBackendError {
    /// Maps this backend error into the domain's [`WindowError`] (spec §2.7).
    ///
    /// Every winit-side failure is a platform failure from the domain's point of
    /// view, so each variant maps to [`WindowError::BackendFailure`] carrying the
    /// human-readable reason; [`WinitBackendError::NoActiveEventLoop`] carries the
    /// fixed message `"no active event loop"`.
    #[must_use]
    pub fn into_window_error(self) -> WindowError {
        match self {
            Self::EventLoopCreation(reason)
            | Self::WindowCreation(reason)
            | Self::Backend(reason) => WindowError::BackendFailure(reason),
            Self::NoActiveEventLoop => {
                WindowError::BackendFailure(Box::from("no active event loop"))
            }
        }
    }
}

impl std::fmt::Display for WinitBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EventLoopCreation(reason) => {
                write!(f, "winit event loop creation failed: {reason}")
            }
            Self::WindowCreation(reason) => {
                write!(f, "winit window creation failed: {reason}")
            }
            Self::NoActiveEventLoop => f.write_str("no active event loop"),
            Self::Backend(reason) => write!(f, "winit backend failure: {reason}"),
        }
    }
}

impl std::error::Error for WinitBackendError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_carry_the_reason() {
        assert_eq!(
            WinitBackendError::EventLoopCreation(Box::from("no display")).to_string(),
            "winit event loop creation failed: no display"
        );
        assert_eq!(
            WinitBackendError::WindowCreation(Box::from("bad size")).to_string(),
            "winit window creation failed: bad size"
        );
        assert_eq!(
            WinitBackendError::NoActiveEventLoop.to_string(),
            "no active event loop"
        );
        assert_eq!(
            WinitBackendError::Backend(Box::from("x11 down")).to_string(),
            "winit backend failure: x11 down"
        );
    }

    #[test]
    fn is_a_std_error() {
        let err = WinitBackendError::NoActiveEventLoop;
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn maps_creation_failures_to_backend_failure() {
        assert_eq!(
            WinitBackendError::EventLoopCreation(Box::from("no display")).into_window_error(),
            WindowError::BackendFailure(Box::from("no display"))
        );
        assert_eq!(
            WinitBackendError::WindowCreation(Box::from("bad size")).into_window_error(),
            WindowError::BackendFailure(Box::from("bad size"))
        );
        assert_eq!(
            WinitBackendError::Backend(Box::from("x11 down")).into_window_error(),
            WindowError::BackendFailure(Box::from("x11 down"))
        );
    }

    #[test]
    fn maps_no_active_event_loop_to_fixed_message() {
        assert_eq!(
            WinitBackendError::NoActiveEventLoop.into_window_error(),
            WindowError::BackendFailure(Box::from("no active event loop"))
        );
    }
}
