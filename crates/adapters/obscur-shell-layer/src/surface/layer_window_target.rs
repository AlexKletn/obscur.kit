//! The raw handles a GPU surface is built from.

use core::ptr::NonNull;

use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle, WindowHandle,
};

use smithay_client_toolkit::shell::{WaylandSurface, wlr_layer::LayerSurface};
use wayland_client::{Connection, Proxy};

/// An owned Wayland target for a renderer. It retains both the layer surface
/// and its display connection, so a GPU surface can keep them alive while
/// it moves across threads independently of the window event loop.
#[derive(Debug, Clone)]
pub struct LayerWindowTarget {
    // Drop the surface role before releasing the display connection.
    layer: LayerSurface,
    connection: Connection,
}

impl LayerWindowTarget {
    /// Retains ownership of the live layer and its associated Wayland connection.
    pub(super) fn new(connection: Connection, layer: LayerSurface) -> Self {
        Self { layer, connection }
    }
}

impl HasDisplayHandle for LayerWindowTarget {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let display = NonNull::new(self.connection.backend().display_ptr().cast())
            .ok_or(HandleError::Unavailable)?;
        let raw = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(display));
        // SAFETY: the target retains the display connection for the entire borrow.
        Ok(unsafe { DisplayHandle::borrow_raw(raw) })
    }
}

impl HasWindowHandle for LayerWindowTarget {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let surface = NonNull::new(self.layer.wl_surface().id().as_ptr().cast())
            .ok_or(HandleError::Unavailable)?;
        let raw = RawWindowHandle::Wayland(WaylandWindowHandle::new(surface));
        // SAFETY: the retained layer owns its wl_surface for the entire borrow.
        Ok(unsafe { WindowHandle::borrow_raw(raw) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::LayerWindow;
    use obscur_windowing::layer::LayerSettings;

    #[test]
    #[ignore = "requires a live Wayland compositor with layer-shell"]
    fn target_keeps_the_layer_alive_after_window_drop() {
        let settings = LayerSettings::new().with_size(1, 1);
        let window = LayerWindow::open(&settings, "obscur-target-lifetime-test").unwrap();
        let target = window.target();
        drop(window);

        std::thread::spawn(move || {
            assert!(target.layer.wl_surface().is_alive());
            assert!(target.window_handle().is_ok());
            assert!(target.display_handle().is_ok());
        })
        .join()
        .expect("renderer thread can use the retained Wayland target");
    }
}
