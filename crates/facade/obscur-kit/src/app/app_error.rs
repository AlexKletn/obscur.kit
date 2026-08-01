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
    #[doc = include_str!("app_error.doc/from.md")]
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
