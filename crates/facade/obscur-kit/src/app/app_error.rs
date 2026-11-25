//! [`AppError`] — why [`App::run`](super::App::run) could not start or complete.

use crate::screen::ScreenError;
#[cfg(feature = "shell-layer")]
use crate::shell_layer::error::LayerShellError;
use crate::view::component::RegistryError;

/// Why [`App::run`](super::App::run) could not start or complete.
#[derive(Debug)]
pub enum AppError {
    /// An `.okui` [`screen`](super::App::screen) did not parse; carries one message
    /// per parse error.
    ScreenParse(Vec<String>),
    /// An `.okui` [`screen`](super::App::screen) parsed but failed to compile;
    /// carries one message per compile diagnostic.
    ScreenCompile(Vec<String>),
    /// A component could not be registered during startup.
    ComponentRegistration(RegistryError),
    /// The winit event loop could not be built or run.
    #[cfg(feature = "window-winit")]
    EventLoop(winit::error::EventLoopError),
    /// The requested window configuration is invalid.
    Window(String),
    /// The OS window could not be created.
    #[cfg(feature = "window-winit")]
    WindowCreation(winit::error::OsError),
    /// No usable GPU adapter/device/surface (as in a headless environment).
    Gpu(String),
    /// The GPU backend could not create a presentation surface.
    GpuSurface(wgpu::CreateSurfaceError),
    /// No compatible GPU adapter could be selected.
    GpuAdapter(wgpu::RequestAdapterError),
    /// The selected adapter could not provide a device.
    GpuDevice(wgpu::RequestDeviceError),
    /// The renderer could not initialize with the selected GPU device.
    GpuRenderer(obscur_render_vello::error::VelloRenderError),
    /// The OS refused to start the GPU surface acquisition thread.
    SurfaceAcquireThread(std::io::Error),
    #[doc = include_str!("../../docs/api/app/app_error.doc/LayerUnsupported.md")]
    LayerUnsupported,
    /// The platform has desktop surfaces, and this one could not be created.
    #[cfg(feature = "shell-layer")]
    Layer(LayerShellError),
    #[doc = include_str!("../../docs/api/app/app_error.doc/NoWindowBackend.md")]
    NoWindowBackend,
    #[doc = include_str!("../../docs/api/app/app_error.doc/FrameAwareLayerRoot.md")]
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
            Self::ComponentRegistration(error) => {
                write!(f, "could not register an application component: {error}")
            }
            #[cfg(feature = "window-winit")]
            Self::EventLoop(error) => write!(f, "window event loop failed: {error}"),
            Self::Window(reason) => write!(f, "could not create the window: {reason}"),
            #[cfg(feature = "window-winit")]
            Self::WindowCreation(error) => write!(f, "could not create the window: {error}"),
            Self::Gpu(reason) => write!(f, "no usable GPU adapter/device/surface: {reason}"),
            Self::GpuSurface(error) => write!(f, "could not create a GPU surface: {error}"),
            Self::GpuAdapter(error) => write!(f, "could not select a GPU adapter: {error}"),
            Self::GpuDevice(error) => write!(f, "could not request a GPU device: {error}"),
            Self::GpuRenderer(error) => write!(f, "could not initialize the GPU renderer: {error}"),
            Self::SurfaceAcquireThread(error) => {
                write!(f, "could not start surface acquisition: {error}")
            }
            Self::LayerUnsupported => f.write_str(
                "this desktop has no layer surfaces, so `App::layer` cannot open a panel \
                 (expected on GNOME, and on any platform without a desktop-surface backend)",
            ),
            #[cfg(feature = "shell-layer")]
            Self::Layer(error) => write!(f, "could not create the layer surface: {error}"),
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

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ComponentRegistration(error) => Some(error),
            Self::SurfaceAcquireThread(error) => Some(error),
            Self::GpuSurface(error) => Some(error),
            Self::GpuAdapter(error) => Some(error),
            Self::GpuDevice(error) => Some(error),
            Self::GpuRenderer(error) => Some(error),
            #[cfg(feature = "window-winit")]
            Self::EventLoop(error) => Some(error),
            #[cfg(feature = "window-winit")]
            Self::WindowCreation(error) => Some(error),
            #[cfg(feature = "shell-layer")]
            Self::Layer(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ScreenError> for AppError {
    /// Carries a screen front-end failure into the one error type a windowed
    /// application sees, keeping the parse/compile split (and the diagnostics)
    /// intact — the `.okui` front end is display-free and has its own error type, so
    /// this is the only place the two meet.
    fn from(error: ScreenError) -> Self {
        match error {
            ScreenError::Parse(messages) => Self::ScreenParse(messages),
            ScreenError::Compile(messages) => Self::ScreenCompile(messages),
        }
    }
}

#[cfg(feature = "shell-layer")]
impl From<LayerShellError> for AppError {
    /// Keeps the one distinction that matters: *this platform cannot*, as against
    /// *this attempt failed*.
    fn from(error: LayerShellError) -> Self {
        match error {
            LayerShellError::Unsupported => Self::LayerUnsupported,
            other => Self::Layer(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io;

    use super::{AppError, ScreenError};

    #[test]
    fn a_surface_acquisition_startup_failure_keeps_the_io_error() {
        let error = AppError::SurfaceAcquireThread(io::Error::from(io::ErrorKind::WouldBlock));
        let kind = error
            .source()
            .and_then(|source| source.downcast_ref::<io::Error>())
            .map(io::Error::kind);

        assert_eq!(kind, Some(io::ErrorKind::WouldBlock));
    }

    #[cfg(feature = "window-winit")]
    #[test]
    fn an_event_loop_failure_keeps_the_winit_error() {
        use winit::error::EventLoopError;

        let error = AppError::EventLoop(EventLoopError::RecreationAttempt);
        let source = error
            .source()
            .and_then(|source| source.downcast_ref::<EventLoopError>());

        assert!(matches!(source, Some(EventLoopError::RecreationAttempt)));
    }

    #[test]
    fn a_renderer_startup_failure_keeps_the_full_error_chain() {
        use obscur_render_vello::error::VelloRenderError;

        let error = AppError::GpuRenderer(VelloRenderError::RendererInitialization(
            vello::Error::NoCompatibleDevice,
        ));
        let backend_error = error
            .source()
            .and_then(|source| source.downcast_ref::<VelloRenderError>());
        let vello_error = backend_error
            .and_then(Error::source)
            .and_then(|source| source.downcast_ref::<vello::Error>());

        assert!(matches!(
            vello_error,
            Some(vello::Error::NoCompatibleDevice)
        ));
    }

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

    #[cfg(feature = "shell-layer")]
    #[test]
    fn a_layer_surface_failure_preserves_the_adapter_error() {
        use super::LayerShellError;

        let error = AppError::from(LayerShellError::Dispatch("broken pipe".into()));
        let source = error
            .source()
            .and_then(|source| source.downcast_ref::<LayerShellError>());

        assert!(
            matches!(source, Some(LayerShellError::Dispatch(reason)) if reason == "broken pipe")
        );
        assert!(matches!(
            AppError::from(LayerShellError::Unsupported),
            AppError::LayerUnsupported
        ));
    }

    #[test]
    fn a_screen_failure_keeps_its_half_and_its_diagnostics() {
        let parse: AppError = ScreenError::Parse(vec!["unexpected `<`".to_owned()]).into();
        assert!(matches!(parse, AppError::ScreenParse(messages) if messages.len() == 1));
        let compile: AppError = ScreenError::Compile(vec!["no slot name".to_owned()]).into();
        assert!(matches!(compile, AppError::ScreenCompile(messages) if messages.len() == 1));
    }
}
