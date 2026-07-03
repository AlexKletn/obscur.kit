//! Desired, mutable properties of a window (spec §6.2).

use obscur_geometry::{Point, Size};

use crate::window::WindowState;

/// The desired presentation properties of a window (spec §6.2).
///
/// These are the caller-authored, mutable properties describing how a window
/// should appear: its title, size, position, presentation state, and a set of
/// boolean flags. They form the *intent*; a backend reconciles it against the
/// real platform window.
///
/// Blur/backdrop effects from §6.2 are intentionally omitted here: they are a
/// capability-gated effect owned by the backend port in a later sub-step, not a
/// plain property of the window model.
///
/// The multiple boolean flags mirror the independent presentation toggles
/// enumerated in spec §6.2 and are deliberately kept as flat fields rather than
/// folded into a bitflag type.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq)]
pub struct WindowProperties {
    /// The window title shown in its title bar and task switchers.
    pub title: Box<str>,
    /// The desired content size in logical pixels.
    pub size: Size,
    /// The desired top-left position, or `None` to let the platform choose.
    pub position: Option<Point>,
    /// The desired presentation state (normal, minimized, maximized, fullscreen).
    pub state: WindowState,
    /// Whether the user may resize the window.
    pub resizable: bool,
    /// Whether the window has server-side decorations (title bar, borders).
    pub decorated: bool,
    /// Whether the window stays above ordinary windows in the z-order.
    pub always_on_top: bool,
    /// Whether the window surface is transparent (allows per-pixel alpha).
    pub transparent: bool,
}

impl WindowProperties {
    /// Sets the title, returning the updated properties.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<Box<str>>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the desired size, returning the updated properties.
    #[must_use]
    pub fn with_size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Sets an explicit desired position, returning the updated properties.
    #[must_use]
    pub fn with_position(mut self, position: Point) -> Self {
        self.position = Some(position);
        self
    }

    /// Sets the desired presentation state, returning the updated properties.
    #[must_use]
    pub fn with_state(mut self, state: WindowState) -> Self {
        self.state = state;
        self
    }

    /// Sets whether the window is resizable, returning the updated properties.
    #[must_use]
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Sets whether the window is decorated, returning the updated properties.
    #[must_use]
    pub fn decorated(mut self, decorated: bool) -> Self {
        self.decorated = decorated;
        self
    }

    /// Sets whether the window is always on top, returning the updated properties.
    #[must_use]
    pub fn always_on_top(mut self, always_on_top: bool) -> Self {
        self.always_on_top = always_on_top;
        self
    }

    /// Sets whether the window surface is transparent, returning the updated
    /// properties.
    #[must_use]
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }
}

impl Default for WindowProperties {
    /// A resizable, decorated 800x600 window with an empty title, no explicit
    /// position, and the [`WindowState::Normal`] presentation state.
    fn default() -> Self {
        Self {
            title: Box::from(""),
            size: Size::new(800.0, 600.0),
            position: None,
            state: WindowState::Normal,
            resizable: true,
            decorated: true,
            always_on_top: false,
            transparent: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let p = WindowProperties::default();
        assert_eq!(&*p.title, "");
        assert_eq!(p.size, Size::new(800.0, 600.0));
        assert_eq!(p.position, None);
        assert_eq!(p.state, WindowState::Normal);
        assert!(p.resizable);
        assert!(p.decorated);
        assert!(!p.always_on_top);
        assert!(!p.transparent);
    }

    #[test]
    fn builder_chaining() {
        let p = WindowProperties::default()
            .with_title("Editor")
            .with_size(Size::new(1024.0, 768.0))
            .with_position(Point::new(10.0, 20.0))
            .with_state(WindowState::Maximized)
            .resizable(false)
            .decorated(false)
            .always_on_top(true)
            .transparent(true);

        assert_eq!(&*p.title, "Editor");
        assert_eq!(p.size, Size::new(1024.0, 768.0));
        assert_eq!(p.position, Some(Point::new(10.0, 20.0)));
        assert_eq!(p.state, WindowState::Maximized);
        assert!(!p.resizable);
        assert!(!p.decorated);
        assert!(p.always_on_top);
        assert!(p.transparent);
    }
}
