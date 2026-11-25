/// Device or input stream that produced a gesture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureSource {
    /// Direct contacts on the display.
    Touchscreen,
    /// Native gesture reported by a touchpad backend.
    Touchpad,
    /// Swipe inferred from phased pixel scrolling. The device and contact count are unspecified.
    PhasedScroll,
}
