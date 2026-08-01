#![doc = include_str!("update_window.doc/_module.md")]

use obscur_geometry::{Point, Size};
use obscur_identity::WindowId;

use crate::window::{WindowProperties, WindowState};

#[doc = include_str!("update_window.doc/UpdateWindow.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateWindow {
    #[doc = include_str!("update_window.doc/target.md")]
    target: WindowId,
    #[doc = include_str!("update_window.doc/title.md")]
    title: Option<Box<str>>,
    #[doc = include_str!("update_window.doc/size.md")]
    size: Option<Size>,
    #[doc = include_str!("update_window.doc/position.md")]
    #[allow(clippy::option_option)]
    position: Option<Option<Point>>,
    #[doc = include_str!("update_window.doc/state.md")]
    state: Option<WindowState>,
    #[doc = include_str!("update_window.doc/resizable.md")]
    resizable: Option<bool>,
    #[doc = include_str!("update_window.doc/decorated.md")]
    decorated: Option<bool>,
    #[doc = include_str!("update_window.doc/always_on_top.md")]
    always_on_top: Option<bool>,
    #[doc = include_str!("update_window.doc/transparent.md")]
    transparent: Option<bool>,
}

impl UpdateWindow {
    #[doc = include_str!("update_window.doc/new.md")]
    #[must_use]
    pub const fn new(target: WindowId) -> Self {
        Self {
            target,
            title: None,
            size: None,
            position: None,
            state: None,
            resizable: None,
            decorated: None,
            always_on_top: None,
            transparent: None,
        }
    }

    #[doc = include_str!("update_window.doc/target_2.md")]
    #[must_use]
    pub const fn target(&self) -> WindowId {
        self.target
    }

    #[doc = include_str!("update_window.doc/set_title.md")]
    #[must_use]
    pub fn set_title(mut self, title: impl Into<Box<str>>) -> Self {
        self.title = Some(title.into());
        self
    }

    #[doc = include_str!("update_window.doc/set_size.md")]
    #[must_use]
    pub fn set_size(mut self, size: Size) -> Self {
        self.size = Some(size);
        self
    }

    #[doc = include_str!("update_window.doc/set_position.md")]
    #[must_use]
    pub fn set_position(mut self, position: Option<Point>) -> Self {
        self.position = Some(position);
        self
    }

    #[doc = include_str!("update_window.doc/set_state.md")]
    #[must_use]
    pub fn set_state(mut self, state: WindowState) -> Self {
        self.state = Some(state);
        self
    }

    #[doc = include_str!("update_window.doc/set_resizable.md")]
    #[must_use]
    pub fn set_resizable(mut self, resizable: bool) -> Self {
        self.resizable = Some(resizable);
        self
    }

    #[doc = include_str!("update_window.doc/set_decorated.md")]
    #[must_use]
    pub fn set_decorated(mut self, decorated: bool) -> Self {
        self.decorated = Some(decorated);
        self
    }

    #[doc = include_str!("update_window.doc/set_always_on_top.md")]
    #[must_use]
    pub fn set_always_on_top(mut self, always_on_top: bool) -> Self {
        self.always_on_top = Some(always_on_top);
        self
    }

    #[doc = include_str!("update_window.doc/always_on_top_2.md")]
    #[must_use]
    pub const fn always_on_top(&self) -> Option<bool> {
        self.always_on_top
    }

    #[doc = include_str!("update_window.doc/set_transparent.md")]
    #[must_use]
    pub fn set_transparent(mut self, transparent: bool) -> Self {
        self.transparent = Some(transparent);
        self
    }

    #[doc = include_str!("update_window.doc/is_empty.md")]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.size.is_none()
            && self.position.is_none()
            && self.state.is_none()
            && self.resizable.is_none()
            && self.decorated.is_none()
            && self.always_on_top.is_none()
            && self.transparent.is_none()
    }

    #[doc = include_str!("update_window.doc/apply.md")]
    pub fn apply(&self, props: &mut WindowProperties) {
        if let Some(title) = &self.title {
            props.title = title.clone();
        }
        if let Some(size) = self.size {
            props.size = size;
        }
        if let Some(position) = self.position {
            props.position = position;
        }
        if let Some(state) = self.state {
            props.state = state;
        }
        if let Some(resizable) = self.resizable {
            props.resizable = resizable;
        }
        if let Some(decorated) = self.decorated {
            props.decorated = decorated;
        }
        if let Some(always_on_top) = self.always_on_top {
            props.always_on_top = always_on_top;
        }
        if let Some(transparent) = self.transparent {
            props.transparent = transparent;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target() -> WindowId {
        WindowId::from_raw(1)
    }

    #[test]
    fn new_is_empty() {
        assert!(UpdateWindow::new(target()).is_empty());
    }

    #[test]
    fn any_setter_makes_non_empty() {
        assert!(!UpdateWindow::new(target()).set_resizable(false).is_empty());
        assert!(!UpdateWindow::new(target()).set_position(None).is_empty());
    }

    #[test]
    fn apply_writes_only_set_fields() {
        let mut props = WindowProperties::default()
            .with_title("Original")
            .with_position(Point::new(1.0, 2.0));
        let delta = UpdateWindow::new(target())
            .set_size(Size::new(320.0, 240.0))
            .set_state(WindowState::Maximized)
            .set_transparent(true);
        delta.apply(&mut props);

        // Changed fields.
        assert_eq!(props.size, Size::new(320.0, 240.0));
        assert_eq!(props.state, WindowState::Maximized);
        assert!(props.transparent);
        // Untouched fields.
        assert_eq!(&*props.title, "Original");
        assert_eq!(props.position, Some(Point::new(1.0, 2.0)));
        assert!(props.resizable);
        assert!(props.decorated);
    }

    #[test]
    fn apply_title_is_written() {
        let mut props = WindowProperties::default().with_title("Old");
        UpdateWindow::new(target())
            .set_title("New")
            .apply(&mut props);
        assert_eq!(&*props.title, "New");
    }

    #[test]
    fn position_outer_none_leaves_unchanged() {
        let mut props = WindowProperties::default().with_position(Point::new(7.0, 8.0));
        // No set_position call: outer None must leave the position alone.
        UpdateWindow::new(target())
            .set_size(Size::new(10.0, 10.0))
            .apply(&mut props);
        assert_eq!(props.position, Some(Point::new(7.0, 8.0)));
    }

    #[test]
    fn position_inner_none_clears_to_platform_chosen() {
        let mut props = WindowProperties::default().with_position(Point::new(7.0, 8.0));
        UpdateWindow::new(target())
            .set_position(None)
            .apply(&mut props);
        assert_eq!(props.position, None);
    }

    #[test]
    fn position_inner_some_sets_explicit() {
        let mut props = WindowProperties::default();
        UpdateWindow::new(target())
            .set_position(Some(Point::new(3.0, 4.0)))
            .apply(&mut props);
        assert_eq!(props.position, Some(Point::new(3.0, 4.0)));
    }
}
