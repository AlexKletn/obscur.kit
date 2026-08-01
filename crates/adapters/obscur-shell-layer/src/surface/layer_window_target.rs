#![doc = include_str!("layer_window_target.doc/_module.md")]

use core::ptr::NonNull;

use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle, WindowHandle,
};

#[doc = include_str!("layer_window_target.doc/LayerWindowTarget.md")]
#[derive(Debug, Clone, Copy)]
pub struct LayerWindowTarget {
    display: NonNull<core::ffi::c_void>,
    surface: NonNull<core::ffi::c_void>,
}

// SAFETY: the two pointers are Wayland proxy addresses, which the client library
// treats as thread-safe handles; the objects they name outlive this value by the
// contract above. Nothing here dereferences them — they are passed straight back
// to wgpu, which does.
unsafe impl Send for LayerWindowTarget {}
unsafe impl Sync for LayerWindowTarget {}

impl LayerWindowTarget {
    #[doc = include_str!("layer_window_target.doc/new.md")]
    #[must_use]
    pub const unsafe fn new(
        display: NonNull<core::ffi::c_void>,
        surface: NonNull<core::ffi::c_void>,
    ) -> Self {
        Self { display, surface }
    }
}

impl HasDisplayHandle for LayerWindowTarget {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let raw = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(self.display));
        // SAFETY: see the type's contract — the display outlives this borrow.
        Ok(unsafe { DisplayHandle::borrow_raw(raw) })
    }
}

impl HasWindowHandle for LayerWindowTarget {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let raw = RawWindowHandle::Wayland(WaylandWindowHandle::new(self.surface));
        // SAFETY: see the type's contract — the surface outlives this borrow.
        Ok(unsafe { WindowHandle::borrow_raw(raw) })
    }
}
