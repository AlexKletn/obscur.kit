#![doc = include_str!("winit_backend_error.doc/_module.md")]

use obscur_windowing::port::WindowError;

#[doc = include_str!("winit_backend_error.doc/WinitBackendError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WinitBackendError {
    #[doc = include_str!("winit_backend_error.doc/EventLoopCreation.md")]
    EventLoopCreation(Box<str>),
    #[doc = include_str!("winit_backend_error.doc/WindowCreation.md")]
    WindowCreation(Box<str>),
    #[doc = include_str!("winit_backend_error.doc/NoActiveEventLoop.md")]
    NoActiveEventLoop,
    #[doc = include_str!("winit_backend_error.doc/Backend.md")]
    Backend(Box<str>),
}

impl WinitBackendError {
    #[doc = include_str!("winit_backend_error.doc/into_window_error.md")]
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
