//! [`AppError`] — why [`App::run`](super::App::run) could not start or complete.

/// Why [`App::run`](super::App::run) could not start or complete.
#[derive(Debug)]
pub enum AppError {
    /// An `.okui` [`screen`](super::App::screen) did not parse; carries one message
    /// per parse error.
    ScreenParse(Vec<String>),
    /// An `.okui` [`screen`](super::App::screen) parsed but failed to compile;
    /// carries one message per compile diagnostic.
    ScreenCompile(Vec<String>),
    /// The winit event loop could not be built.
    EventLoop(String),
    /// The OS window could not be created.
    Window(String),
    /// No usable GPU adapter/device/surface (as in a headless environment).
    Gpu(String),
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
        }
    }
}

impl std::error::Error for AppError {}
