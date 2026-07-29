//! The raw handles a GPU surface is built from.

use core::ptr::NonNull;

use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle, WindowHandle,
};

/// An owned, `'static` pair of Wayland handles, so a host can build a
/// `wgpu::Surface<'static>` from a layer surface exactly as it does from a
/// window.
///
/// # Why this type exists at all
///
/// wgpu takes anything implementing [`HasWindowHandle`] + [`HasDisplayHandle`],
/// and winit's window implements both — which is why the ordinary windowed path
/// needs no unsafe anywhere. A raw `wl_surface` implements neither, so something
/// has to assert the handles are valid. That assertion is the *only* unsafe in
/// this crate, and it is here rather than in the host because the host forbids
/// unsafe outright.
///
/// # Safety contract
///
/// The pointers are borrowed from the connection and the `wl_surface` owned by
/// [`LayerWindow`](super::LayerWindow). This type must not outlive it. It is
/// constructed only by `LayerWindow::target`, which hands out a fresh one from
/// live objects, and it is inert data — dropping it releases nothing.
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
    /// Builds a target from a live display and surface pointer.
    ///
    /// # Safety
    ///
    /// Both pointers must name live Wayland objects that outlive every
    /// `wgpu::Surface` built from this target.
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
