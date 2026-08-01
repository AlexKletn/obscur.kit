#![doc = include_str!("window_properties.doc/_module.md")]

use obscur_geometry::{Point, Size};

use crate::window::WindowState;

#[doc = include_str!("window_properties.doc/WindowProperties.md")]
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq)]
pub struct WindowProperties {
    #[doc = include_str!("window_properties.doc/title.md")]
    pub title: Box<str>,
    #[doc = include_str!("window_properties.doc/size.md")]
    pub size: Size,
    #[doc = include_str!("window_properties.doc/position.md")]
    pub position: Option<Point>,
    #[doc = include_str!("window_properties.doc/state.md")]
    pub state: WindowState,
    #[doc = include_str!("window_properties.doc/resizable.md")]
    pub resizable: bool,
    #[doc = include_str!("window_properties.doc/decorated.md")]
    pub decorated: bool,
    #[doc = include_str!("window_properties.doc/always_on_top.md")]
    pub always_on_top: bool,
    #[doc = include_str!("window_properties.doc/transparent.md")]
    pub transparent: bool,
}

impl WindowProperties {
    #[doc = include_str!("window_properties.doc/with_title.md")]
    #[must_use]
    pub fn with_title(mut self, title: impl Into<Box<str>>) -> Self {
        self.title = title.into();
        self
    }

    #[doc = include_str!("window_properties.doc/with_size.md")]
    #[must_use]
    pub fn with_size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    #[doc = include_str!("window_properties.doc/with_position.md")]
    #[must_use]
    pub fn with_position(mut self, position: Point) -> Self {
        self.position = Some(position);
        self
    }

    #[doc = include_str!("window_properties.doc/with_state.md")]
    #[must_use]
    pub fn with_state(mut self, state: WindowState) -> Self {
        self.state = state;
        self
    }

    #[doc = include_str!("window_properties.doc/resizable_2.md")]
    #[must_use]
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    #[doc = include_str!("window_properties.doc/decorated_2.md")]
    #[must_use]
    pub fn decorated(mut self, decorated: bool) -> Self {
        self.decorated = decorated;
        self
    }

    #[doc = include_str!("window_properties.doc/always_on_top_2.md")]
    #[must_use]
    pub fn always_on_top(mut self, always_on_top: bool) -> Self {
        self.always_on_top = always_on_top;
        self
    }

    #[doc = include_str!("window_properties.doc/transparent_2.md")]
    #[must_use]
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }
}

impl Default for WindowProperties {
    #[doc = include_str!("window_properties.doc/default.md")]
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
