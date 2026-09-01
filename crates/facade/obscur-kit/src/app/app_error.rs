#![doc = include_str!("app_error.doc/_module.md")]

use crate::screen::ScreenError;

#[doc = include_str!("app_error.doc/AppError.md")]
#[derive(Debug)]
pub enum AppError {
    #[doc = include_str!("app_error.doc/ScreenParse.md")]
    ScreenParse(Vec<String>),
    #[doc = include_str!("app_error.doc/ScreenCompile.md")]
    ScreenCompile(Vec<String>),
    #[doc = include_str!("app_error.doc/EventLoop.md")]
    EventLoop(String),
    #[doc = include_str!("app_error.doc/Window.md")]
    Window(String),
    #[doc = include_str!("app_error.doc/Gpu.md")]
    Gpu(String),
    #[doc = include_str!("app_error.doc/LayerUnsupported.md")]
    LayerUnsupported,
    #[doc = include_str!("app_error.doc/Layer.md")]
    Layer(String),
    #[doc = include_str!("app_error.doc/NoWindowBackend.md")]
    NoWindowBackend,
    #[doc = include_str!("app_error.doc/FrameAwareLayerRoot.md")]
    FrameAwareLayerRoot,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ScreenParse(errors) => {
                write!(f, "the .okui screen did not parse: {}", errors.join("; "))
            }
            Self::ScreenCompile(diagnostics) => {
                write!(
                    f,
                    "the .okui screen failed to compile: {}",
                    diagnostics.join("; ")
                )
            }
            Self::EventLoop(reason) => write!(f, "could not build the event loop: {reason}"),
            Self::Window(reason) => write!(f, "could not create the window: {reason}"),
            Self::Gpu(reason) => write!(f, "no usable GPU adapter/device/surface: {reason}"),
            Self::LayerUnsupported => f.write_str(
                "this desktop has no layer surfaces, so `App::layer` cannot open a panel \
                 (expected on GNOME, and on any platform without a desktop-surface backend)",
            ),
            Self::Layer(reason) => write!(f, "could not create the layer surface: {reason}"),
            Self::NoWindowBackend => f.write_str(
                "this build has no windowing backend: enable `window-winit`, or call \
                 `App::layer` to open a desktop surface instead",
            ),
            Self::FrameAwareLayerRoot => f.write_str(
                "`App::view_with` is a windowed root: a layer surface is sized by the \
                 compositor before the first frame, so use `App::view`",
            ),
        }
    }
}

impl std::error::Error for AppError {}

impl From<ScreenError> for AppError {
    #[doc = include_str!("app_error.doc/from.md")]
    fn from(error: ScreenError) -> Self {
        match error {
            ScreenError::Parse(messages) => Self::ScreenParse(messages),
            ScreenError::Compile(messages) => Self::ScreenCompile(messages),
        }
    }
}

#[cfg(feature = "shell-layer")]
impl From<crate::shell_layer::error::LayerShellError> for AppError {
    #[doc = include_str!("app_error.doc/from_layer.md")]
    fn from(error: crate::shell_layer::error::LayerShellError) -> Self {
        match error {
            crate::shell_layer::error::LayerShellError::Unsupported => Self::LayerUnsupported,
            other => Self::Layer(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppError, ScreenError};

    #[test]
    fn the_two_configuration_mistakes_say_what_to_do_instead() {
        // Both are build-time or call-site mistakes rather than platform limits,
        // so the text has to name the fix, not just the fault.
        let no_backend = AppError::NoWindowBackend.to_string();
        assert!(no_backend.contains("window-winit"), "{no_backend}");
        assert!(no_backend.contains("App::layer"), "{no_backend}");
        let frame_aware = AppError::FrameAwareLayerRoot.to_string();
        assert!(frame_aware.contains("App::view"), "{frame_aware}");
    }

    #[test]
    fn an_absent_desktop_surface_is_a_capability_answer() {
        // The one a host is expected to branch on and fall back to a window.
        let text = AppError::LayerUnsupported.to_string();
        assert!(text.contains("no layer surfaces"), "{text}");
    }

    #[test]
    fn a_screen_failure_keeps_its_half_and_its_diagnostics() {
        let parse: AppError = ScreenError::Parse(vec!["unexpected `<`".to_owned()]).into();
        assert!(matches!(parse, AppError::ScreenParse(messages) if messages.len() == 1));
        let compile: AppError = ScreenError::Compile(vec!["no slot name".to_owned()]).into();
        assert!(matches!(compile, AppError::ScreenCompile(messages) if messages.len() == 1));
    }
}
