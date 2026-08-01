#![doc = include_str!("virtual_window.doc/_module.md")]

use obscur_geometry::{ScaleFactor, Size};
use obscur_identity::WindowId;

#[doc = include_str!("virtual_window.doc/VirtualWindow.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualWindow {
    #[doc = include_str!("virtual_window.doc/id.md")]
    id: WindowId,
    #[doc = include_str!("virtual_window.doc/size.md")]
    size: Size,
    #[doc = include_str!("virtual_window.doc/scale.md")]
    scale: ScaleFactor,
    #[doc = include_str!("virtual_window.doc/visible.md")]
    visible: bool,
}

impl VirtualWindow {
    #[doc = include_str!("virtual_window.doc/new.md")]
    #[must_use]
    pub const fn new(id: WindowId, size: Size, scale: ScaleFactor) -> Self {
        Self {
            id,
            size,
            scale,
            visible: true,
        }
    }

    #[doc = include_str!("virtual_window.doc/with_visible.md")]
    #[must_use]
    pub const fn with_visible(self, visible: bool) -> Self {
        Self {
            id: self.id,
            size: self.size,
            scale: self.scale,
            visible,
        }
    }

    #[doc = include_str!("virtual_window.doc/id_2.md")]
    #[must_use]
    pub const fn id(self) -> WindowId {
        self.id
    }

    #[doc = include_str!("virtual_window.doc/size_2.md")]
    #[must_use]
    pub const fn size(self) -> Size {
        self.size
    }

    #[doc = include_str!("virtual_window.doc/scale_2.md")]
    #[must_use]
    pub const fn scale(self) -> ScaleFactor {
        self.scale
    }

    #[doc = include_str!("virtual_window.doc/visible_2.md")]
    #[must_use]
    pub const fn visible(self) -> bool {
        self.visible
    }

    #[doc = include_str!("virtual_window.doc/physical_size.md")]
    #[must_use]
    pub fn physical_size(self) -> Size {
        let factor = self.scale.get();
        Size::new(self.size.width * factor, self.size.height * factor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window() -> VirtualWindow {
        VirtualWindow::new(
            WindowId::from_raw(7),
            Size::new(200.0, 100.0),
            ScaleFactor::new(2.0),
        )
    }

    #[test]
    fn new_is_visible_and_stores_fields() {
        let w = window();
        assert_eq!(w.id(), WindowId::from_raw(7));
        assert_eq!(w.size(), Size::new(200.0, 100.0));
        assert_eq!(w.scale().get(), 2.0);
        assert!(w.visible());
    }

    #[test]
    fn with_visible_toggles_visibility_only() {
        let hidden = window().with_visible(false);
        assert!(!hidden.visible());
        assert_eq!(hidden.size(), Size::new(200.0, 100.0));
        assert_eq!(hidden.id(), WindowId::from_raw(7));
    }

    #[test]
    fn physical_size_multiplies_by_scale() {
        assert_eq!(window().physical_size(), Size::new(400.0, 200.0));
    }

    #[test]
    fn physical_size_at_identity_equals_logical_size() {
        let w = VirtualWindow::new(
            WindowId::from_raw(1),
            Size::new(800.0, 600.0),
            ScaleFactor::IDENTITY,
        );
        assert_eq!(w.physical_size(), w.size());
    }

    #[test]
    fn is_copy_and_eq() {
        let w = window();
        let copy = w;
        assert_eq!(w, copy);
    }
}
