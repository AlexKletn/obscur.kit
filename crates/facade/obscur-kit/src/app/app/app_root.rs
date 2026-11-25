use super::{FrameInfo, View};

/// The root the window renders. The last of [`App::view`](super::App::view) / [`App::view_with`](super::App::view_with) /
/// [`App::screen`](super::App::screen) to be called wins.
pub(super) enum AppRoot {
    /// A reactive builder re-invoked each frame — the same shape
    /// [`UiRuntime::set_root_view`](crate::runtime::UiRuntime::set_root_view) holds.
    View(Box<dyn FnMut() -> View + 'static>),
    /// An `.okui` source, compiled to UI IR when [`App::run`](super::App::run) is called (a compile
    /// failure surfaces as an [`AppError`](super::AppError) from `run`) and lowered to a `View` on
    /// every frame through [`OkuiScreen`](super::OkuiScreen).
    Screen(String),
    /// A builder that additionally reads the frame's [`FrameInfo`].
    ViewWith(Box<dyn FnMut(&FrameInfo) -> View + 'static>),
}
