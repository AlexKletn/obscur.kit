//! Protocol selection and background blur for a live Wayland surface.

mod blur_dispatch;
mod blur_effect;

use blur_dispatch::BlurDispatch;
use blur_effect::BlurEffect;

use wayland_backend::client::WaylandError;
use wayland_client::globals::registry_queue_init;
use wayland_client::protocol::{wl_compositor, wl_region, wl_surface::WlSurface};
use wayland_client::{Connection, Proxy, QueueHandle};
use wayland_protocols::ext::background_effect::v1::client::ext_background_effect_manager_v1::ExtBackgroundEffectManagerV1;
use wayland_protocols_plasma::blur::client::org_kde_kwin_blur_manager::OrgKdeKwinBlurManager;

use crate::rounded_region::RoundedRegion;

/// Keeps the background blur request alive until the surface is closed.
pub struct BackgroundBlur {
    effect: BlurEffect,
    surface: WlSurface,
    compositor: wl_compositor::WlCompositor,
    region: wl_region::WlRegion,
    shape: Option<RoundedRegion>,
    opaque: Option<[i32; 4]>,
    queue: QueueHandle<BlurDispatch>,
    enabled: bool,
    connection: Connection,
}

impl BackgroundBlur {
    /// Requests blur over the full surface. Returns None when blur is unavailable. Call at most once per surface and commit using the surface owner.
    pub fn request(surface: &WlSurface) -> Result<Option<Self>, String> {
        let Some(backend) = surface.backend().upgrade() else {
            return Ok(None);
        };
        let connection = Connection::from_backend(backend);
        let (globals, mut queue) =
            registry_queue_init::<BlurDispatch>(&connection).map_err(|error| error.to_string())?;
        let qh = queue.handle();
        let standard = globals
            .bind::<ExtBackgroundEffectManagerV1, _, _>(&qh, 1..=1, ())
            .ok();
        let mut state = BlurDispatch::default();
        if standard.is_some() {
            queue
                .roundtrip(&mut state)
                .map_err(|error| error.to_string())?;
        }
        let standard = if state.blur {
            standard
        } else {
            if let Some(manager) = standard {
                manager.destroy();
            }
            None
        };
        let kde = (!state.blur)
            .then(|| {
                globals
                    .bind::<OrgKdeKwinBlurManager, _, _>(&qh, 1..=1, ())
                    .ok()
            })
            .flatten();
        if !state.blur && kde.is_none() {
            globals.destroy();
            return Ok(None);
        }
        let compositor = globals
            .bind::<wl_compositor::WlCompositor, _, _>(&qh, 1..=1, ())
            .map_err(|error| error.to_string())?;
        let region = compositor.create_region(&qh, ());
        // The compositor clips this region to the current surface, including after a resize.
        region.add(0, 0, i32::MAX, i32::MAX);
        let effect = if let Some(manager) = standard {
            let effect = manager.get_background_effect(surface, &qh, ());
            manager.destroy();
            BlurEffect::Standard(effect)
        } else if let Some(manager) = kde {
            BlurEffect::Kde {
                manager,
                surface: surface.clone(),
                effect: None,
            }
        } else {
            region.destroy();
            return Ok(None);
        };
        globals.destroy();
        let mut request = Self {
            effect,
            surface: surface.clone(),
            compositor,
            region,
            shape: None,
            opaque: None,
            queue: qh,
            enabled: false,
            connection,
        };
        request.set_enabled(true);
        match request.connection.flush() {
            Ok(()) => Ok(Some(request)),
            Err(WaylandError::Io(error)) if error.kind() == std::io::ErrorKind::WouldBlock => {
                Ok(Some(request))
            }
            Err(error) => Err(error.to_string()),
        }
    }

    /// Returns the compositor protocol selected for this request.
    #[must_use]
    pub const fn protocol(&self) -> &'static str {
        match self.effect {
            BlurEffect::Standard(_) => "ext-background-effect-v1",
            BlurEffect::Kde { .. } => "org_kde_kwin_blur",
        }
    }

    /// Update the region in surface-local coordinates before presenting the frame.
    pub fn set_surface_shape(&mut self, size: [f32; 2], radii: [[f32; 2]; 4]) {
        self.set_surface_occlusion(size, radii, None);
    }

    /// Stages the rounded blur shape and the proven opaque rectangle in logical surface coordinates. Opaque pixels are excluded from blur and reported as an opaque region. Pass `None` when opacity cannot be guaranteed.
    pub fn set_surface_occlusion(
        &mut self,
        size: [f32; 2],
        radii: [[f32; 2]; 4],
        opaque: Option<[i32; 4]>,
    ) {
        let shape = RoundedRegion::new(size, radii);
        if self.shape == shape && self.opaque == opaque {
            return;
        }

        let region = self.compositor.create_region(&self.queue, ());
        if let Some(shape) = shape {
            shape.rectangles(|x, y, width, height| region.add(x, y, width, height));
        } else {
            region.add(0, 0, i32::MAX, i32::MAX);
        }
        if let Some([x, y, width, height]) = opaque {
            region.subtract(x, y, width, height);
        }
        if self.opaque != opaque {
            if let Some([x, y, width, height]) = opaque {
                let opaque_region = self.compositor.create_region(&self.queue, ());
                opaque_region.add(x, y, width, height);
                self.surface.set_opaque_region(Some(&opaque_region));
                opaque_region.destroy();
            } else {
                self.surface.set_opaque_region(None);
            }
        }
        if self.enabled {
            match &self.effect {
                BlurEffect::Standard(effect) => effect.set_blur_region(Some(&region)),
                BlurEffect::Kde { effect, .. } => {
                    if let Some(blur) = effect {
                        blur.set_region(Some(&region));
                        blur.commit();
                    }
                }
            }
        }
        std::mem::replace(&mut self.region, region).destroy();
        self.shape = shape;
        self.opaque = opaque;
    }

    /// Stages an enable or disable for the next surface commit.
    pub fn set_enabled(&mut self, enabled: bool) {
        if self.enabled == enabled {
            return;
        }
        match &mut self.effect {
            BlurEffect::Standard(effect) => effect.set_blur_region(enabled.then_some(&self.region)),
            BlurEffect::Kde {
                manager,
                surface,
                effect,
            } => {
                if enabled {
                    let blur = manager.create(surface, &self.queue, ());
                    blur.set_region(Some(&self.region));
                    blur.commit();
                    *effect = Some(blur);
                } else if let Some(blur) = effect.take() {
                    manager.unset(surface);
                    blur.release();
                }
            }
        }
        self.enabled = enabled;
    }
}

impl Drop for BackgroundBlur {
    fn drop(&mut self) {
        if let BlurEffect::Standard(effect) = &self.effect {
            effect.destroy();
        } else {
            self.set_enabled(false);
        }
        self.region.destroy();
        let _ = self.connection.flush();
    }
}
