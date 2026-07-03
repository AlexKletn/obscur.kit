//! A windowless mapping of a window: [`VirtualWindow`].

use obscur_geometry::{ScaleFactor, Size};
use obscur_identity::WindowId;

/// A windowless stand-in for a window (spec §6.1).
///
/// On the headless and WASM backends there is no OS window and no native
/// surface, so every window concept maps to a `VirtualWindow`: a plain record of
/// the [`WindowId`], the logical `size`, the display `scale` (DPI), and whether
/// the window is `visible`. It owns no OS resource and never touches the
/// platform — it exists only so the rest of the stack can reason about "a
/// window" uniformly across backends.
///
/// The type is a small plain value (`Copy`), so a host can hold it by value and
/// hand copies out freely.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualWindow {
    /// The identity of this virtual window.
    id: WindowId,
    /// The logical size of the window in logical pixels.
    size: Size,
    /// The display scale factor mapping logical to physical pixels.
    scale: ScaleFactor,
    /// Whether the window is currently visible.
    visible: bool,
}

impl VirtualWindow {
    /// Creates a visible virtual window with the given identity, logical `size`,
    /// and display `scale`.
    #[must_use]
    pub const fn new(id: WindowId, size: Size, scale: ScaleFactor) -> Self {
        Self {
            id,
            size,
            scale,
            visible: true,
        }
    }

    /// Returns a copy of this window with `visible` replaced.
    #[must_use]
    pub const fn with_visible(self, visible: bool) -> Self {
        Self {
            id: self.id,
            size: self.size,
            scale: self.scale,
            visible,
        }
    }

    /// Returns the identity of this virtual window.
    #[must_use]
    pub const fn id(self) -> WindowId {
        self.id
    }

    /// Returns the logical size of the window.
    #[must_use]
    pub const fn size(self) -> Size {
        self.size
    }

    /// Returns the display scale factor mapping logical to physical pixels.
    #[must_use]
    pub const fn scale(self) -> ScaleFactor {
        self.scale
    }

    /// Returns whether the window is currently visible.
    #[must_use]
    pub const fn visible(self) -> bool {
        self.visible
    }

    /// Returns the physical size of the window: the logical [`size`](Self::size)
    /// multiplied by the [`scale`](Self::scale) factor on each axis.
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
