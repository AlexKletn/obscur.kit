#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum PacingMode {
    #[default]
    Feedback,
    Timer,
    Swapchain,
    Display,
}
