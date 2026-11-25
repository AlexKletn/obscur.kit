//! Attaches background blur to a toolkit-owned Wayland window.

use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use wayland_backend::client::{Backend, ObjectId};
use wayland_client::{Connection, Proxy, protocol::wl_surface::WlSurface};

use crate::BackgroundBlur;

/// Keeps a blur request and its toolkit-owned window alive.
pub struct WindowBlur<W> {
    effect: BackgroundBlur,
    _window: W,
}

impl<W: HasDisplayHandle + HasWindowHandle> WindowBlur<W> {
    /// Returns the compositor protocol selected for this request.
    #[must_use]
    pub const fn protocol(&self) -> &'static str {
        self.effect.protocol()
    }

    /// Stages an enable or disable for the next surface commit.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.effect.set_enabled(enabled);
    }

    /// Update the blur shape to match the rendered window.
    pub fn set_surface_shape(&mut self, size: [f32; 2], radii: [[f32; 2]; 4]) {
        self.effect.set_surface_shape(size, radii);
    }

    /// Stages the rounded blur shape and the proven opaque rectangle in logical surface coordinates. Opaque pixels are excluded from blur and reported as an opaque region. Pass `None` when opacity cannot be guaranteed.
    pub fn set_surface_occlusion(
        &mut self,
        size: [f32; 2],
        radii: [[f32; 2]; 4],
        opaque: Option<[i32; 4]>,
    ) {
        self.effect.set_surface_occlusion(size, radii, opaque);
    }

    /// Requests background blur for a Wayland window. Returns None on other platforms or when the compositor does not offer blur.
    pub fn request(window: W) -> Result<Option<Self>, String> {
        // SAFETY: the stored window outlives the blur effect and its proxies.
        let Some((_connection, surface)) = (unsafe { window_surface(&window) })? else {
            return Ok(None);
        };
        Ok(BackgroundBlur::request(&surface)?.map(|effect| Self {
            effect,
            _window: window,
        }))
    }
}

#[doc = include_str!("../docs/api/window_blur.doc/window_surface.md")]
pub(crate) unsafe fn window_surface<W: HasDisplayHandle + HasWindowHandle>(
    window: &W,
) -> Result<Option<(Connection, WlSurface)>, String> {
    let display = window.display_handle().map_err(|error| error.to_string())?;
    let handle = window.window_handle().map_err(|error| error.to_string())?;
    let (RawDisplayHandle::Wayland(display), RawWindowHandle::Wayland(handle)) =
        (display.as_raw(), handle.as_raw())
    else {
        return Ok(None);
    };

    // SAFETY: the handles borrow the same live window. The guest backend owns
    // its queue but does not close the display. Callers keep the window alive
    // for the lifetime of the connection and surface.
    let backend = unsafe { Backend::from_foreign_display(display.display.as_ptr().cast()) };
    let connection = Connection::from_backend(backend);
    // SAFETY: the Wayland window handle guarantees a live wl_surface proxy.
    let id = unsafe { ObjectId::from_ptr(WlSurface::interface(), handle.surface.as_ptr().cast()) }
        .map_err(|error| error.to_string())?;
    let surface = WlSurface::from_id(&connection, id).map_err(|error| error.to_string())?;
    Ok(Some((connection, surface)))
}
