//! [`AppError`] — why [`App::run`](super::App::run) could not start or complete.

use crate::screen::ScreenError;

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

#[cfg(test)]
mod tests {
    use super::{AppError, ScreenError};

    #[test]
    fn a_screen_failure_keeps_its_half_and_its_diagnostics() {
        let parse: AppError = ScreenError::Parse(vec!["unexpected `<`".to_owned()]).into();
        assert!(matches!(parse, AppError::ScreenParse(messages) if messages.len() == 1));
        let compile: AppError = ScreenError::Compile(vec!["no slot name".to_owned()]).into();
        assert!(matches!(compile, AppError::ScreenCompile(messages) if messages.len() == 1));
    }
}
