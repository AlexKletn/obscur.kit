#![doc = include_str!("layer_window.doc/_module.md")]

use core::ptr::NonNull;

use calloop::EventLoop;
use calloop_wayland_source::WaylandSource;
use obscur_capability::SupportLevel;
use obscur_geometry::Size;
use obscur_windowing::layer::{
    LayerDisplay, LayerPointerPolicy, LayerSettings, LayerSurfaceEvent, LayerSurfaceState,
};
use obscur_windowing::port::{LayerBackend, LayerError, LayerFeature};
use obscur_windowing::request::WindowOperationResult;
use smithay_client_toolkit::compositor::{CompositorState, Region};
use smithay_client_toolkit::output::OutputState;
use smithay_client_toolkit::registry::RegistryState;
use smithay_client_toolkit::seat::SeatState;
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::shell::wlr_layer::{LayerShell, LayerSurface};
use wayland_client::globals::registry_queue_init;
use wayland_client::{Connection, Proxy};
use wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1;
use wayland_protocols::wp::presentation_time::client::wp_presentation::WpPresentation;
use wayland_protocols::wp::viewporter::client::wp_viewporter::WpViewporter;

use crate::error::LayerShellError;
use crate::event::{LayerInput, Presented};

use super::layer_state::LayerState;
use super::layer_waker::LayerWaker;
use super::layer_window_target::LayerWindowTarget;
use super::wlr_mapper::WlrMapper;

#[doc = include_str!("layer_window.doc/LayerWindow.md")]
pub struct LayerWindow {
    connection: Connection,
    event_loop: EventLoop<'static, LayerState>,
    state: LayerState,
    #[doc = include_str!("layer_window.doc/settings.md")]
    settings: LayerSettings,
    #[doc = include_str!("layer_window.doc/compositor.md")]
    compositor: CompositorState,
    #[doc = include_str!("layer_window.doc/_fractional_manager.md")]
    _fractional_manager: Option<WpFractionalScaleManagerV1>,
    _viewporter: Option<WpViewporter>,
    #[doc = include_str!("layer_window.doc/queue.md")]
    queue: wayland_client::QueueHandle<LayerState>,
}

impl LayerWindow {
    #[doc = include_str!("layer_window.doc/open.md")]
    pub fn open(settings: &LayerSettings, namespace: &str) -> Result<Self, LayerShellError> {
        Self::open_with(settings, namespace, None)
    }

    #[doc = include_str!("layer_window.doc/open_with.md")]
    pub fn open_with(
        settings: &LayerSettings,
        namespace: &str,
        waker: Option<&LayerWaker>,
    ) -> Result<Self, LayerShellError> {
        let connection = Connection::connect_to_env()
            .map_err(|error| LayerShellError::NoDisplay(error.to_string()))?;
        let (globals, queue) = registry_queue_init::<LayerState>(&connection)
            .map_err(|error| LayerShellError::Dispatch(error.to_string()))?;
        let qh = queue.handle();

        let compositor = CompositorState::bind(&globals, &qh)
            .map_err(|_| LayerShellError::MissingGlobal("wl_compositor".to_owned()))?;
        let shell = LayerShell::bind(&globals, &qh).map_err(|_| LayerShellError::Unsupported)?;

        // Outputs must be enumerated before the surface can name one, so the
        // state exists first and the surface is created into it. One roundtrip
        // is what turns the registry's bare globals into identified displays.
        let mut state = LayerState::new(
            RegistryState::new(&globals),
            SeatState::new(&globals, &qh),
            OutputState::new(&globals, &qh),
            initial_size(settings),
        );
        let mut queue = queue;
        queue
            .roundtrip(&mut state)
            .map_err(|error| LayerShellError::Dispatch(error.to_string()))?;

        let chosen = settings.display.and_then(|id| state.output_with_id(id));

        let surface = compositor.create_surface(&qh);
        let layer = shell.create_layer_surface(
            &qh,
            surface,
            WlrMapper::layer(settings.band),
            Some(namespace.to_owned()),
            chosen.as_ref(),
        );
        apply(&compositor, &layer, settings);

        // Fractional scaling, where the compositor offers it. Both halves are
        // needed: the fractional-scale object reports a scale in 120ths, and the
        // viewport is what maps a buffer rendered at that scale back onto the
        // logical size. With only one of the two the surface would be the wrong
        // size, so it is both or neither.
        let fractional_manager = globals
            .bind::<WpFractionalScaleManagerV1, _, _>(&qh, 1..=1, ())
            .ok();
        let viewporter = globals.bind::<WpViewporter, _, _>(&qh, 1..=1, ()).ok();
        // Optional on purpose: a compositor without `wp_presentation` loses the
        // timings and keeps the surface. Nothing below this line may depend on
        // it existing.
        state.presentation = globals.bind::<WpPresentation, _, _>(&qh, 1..=2, ()).ok();
        if state.presentation.is_none() {
            eprintln!(
                "obscur-shell-layer: no wp_presentation here; \
                 frames cannot be timed to the screen"
            );
        }
        if let (Some(manager), Some(viewporter)) = (&fractional_manager, &viewporter) {
            state.fractional = Some(manager.get_fractional_scale(layer.wl_surface(), &qh, ()));
            state.viewport = Some(viewporter.get_viewport(layer.wl_surface(), &qh, ()));
        }

        // The initial commit with no buffer is what asks for the first configure.
        layer.commit();
        state.layer = Some(layer);

        // The loop is built only after the roundtrip above, because a
        // `WaylandSource` consumes the queue and the outputs must be enumerated
        // first.
        let event_loop: EventLoop<'static, LayerState> =
            EventLoop::try_new().map_err(|error| LayerShellError::Dispatch(error.to_string()))?;
        WaylandSource::new(connection.clone(), queue)
            .insert(event_loop.handle())
            .map_err(|error| LayerShellError::Dispatch(error.to_string()))?;
        if let Some(source) = waker.and_then(LayerWaker::take_source) {
            event_loop
                .handle()
                .insert_source(source, |(), (), state: &mut LayerState| {
                    // The turn alone is not enough: a host renders when it has a
                    // reason to, so the wake has to become one.
                    state.surface_events.push(LayerSurfaceEvent::Woken);
                })
                .map_err(|error| LayerShellError::Dispatch(error.to_string()))?;
        }

        Ok(Self {
            connection,
            event_loop,
            state,
            settings: settings.clone(),
            compositor,
            _fractional_manager: fractional_manager,
            _viewporter: viewporter,
            queue: qh,
        })
    }

    #[doc = include_str!("layer_window.doc/dispatch.md")]
    pub fn dispatch(
        &mut self,
        timeout: Option<std::time::Duration>,
    ) -> Result<(), LayerShellError> {
        self.event_loop
            .dispatch(timeout, &mut self.state)
            .map_err(|error| LayerShellError::Dispatch(error.to_string()))
    }

    #[doc = include_str!("layer_window.doc/poll_input.md")]
    pub fn poll_input(&mut self) -> Vec<LayerInput> {
        core::mem::take(&mut self.state.input)
    }

    #[doc = include_str!("layer_window.doc/ask_when_shown.md")]
    pub fn ask_when_shown(&mut self) {
        let (Some(presentation), Some(layer)) = (&self.state.presentation, &self.state.layer)
        else {
            return;
        };
        let seq = self.state.feedbacks_asked;
        self.state.feedbacks_asked += 1;
        // Bound to the surface's *next* commit, which is the one wgpu is about
        // to make. Asked before presenting rather than after, because after is
        // the commit following the one being measured.
        presentation.feedback(layer.wl_surface(), &self.queue, seq);
    }

    #[doc = include_str!("layer_window.doc/take_presented.md")]
    pub fn take_presented(&mut self) -> Vec<Presented> {
        core::mem::take(&mut self.state.presented)
    }

    #[doc = include_str!("layer_window.doc/add_turn_waker.md")]
    pub fn add_turn_waker(&self, waker: &LayerWaker) -> Result<(), LayerShellError> {
        let Some(source) = waker.take_source() else {
            return Ok(());
        };
        self.event_loop
            .handle()
            .insert_source(source, |(), (), _state: &mut LayerState| {
                // Deliberately empty. Ringing this means "your `dispatch` may
                // return now", not "something changed" — the caller looks at
                // whatever it was waiting for. Pushing a `Woken` here would tell
                // the host to reconsider the interface, which is a different
                // claim and would cost a frame nobody asked for.
            })
            .map(|_| ())
            .map_err(|error| LayerShellError::Dispatch(error.to_string()))
    }

    #[doc = include_str!("layer_window.doc/loop_handle.md")]
    #[must_use]
    pub fn loop_handle(&self) -> calloop::LoopHandle<'static, LayerState> {
        self.event_loop.handle()
    }

    #[doc = include_str!("layer_window.doc/flush.md")]
    pub fn flush(&self) -> Result<(), LayerShellError> {
        self.connection
            .flush()
            .map_err(|error| LayerShellError::Dispatch(error.to_string()))
    }

    #[doc = include_str!("layer_window.doc/is_configured.md")]
    #[must_use]
    pub const fn is_configured(&self) -> bool {
        self.state.configured
    }

    #[doc = include_str!("layer_window.doc/is_closed.md")]
    #[must_use]
    pub const fn is_closed(&self) -> bool {
        self.state.closed
    }

    #[doc = include_str!("layer_window.doc/settings_fn.md")]
    #[must_use]
    pub const fn settings(&self) -> &LayerSettings {
        &self.settings
    }

    #[doc = include_str!("layer_window.doc/displays.md")]
    #[must_use]
    pub fn displays(&self) -> Vec<LayerDisplay> {
        self.state.displays()
    }

    #[doc = include_str!("layer_window.doc/sync_scale.md")]
    pub fn sync_scale(&self) {
        let logical = self.state.size;
        if let Some(viewport) = self.state.viewport.as_ref() {
            self.state.layer().wl_surface().set_buffer_scale(1);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            viewport.set_destination(
                (logical.width.round() as i32).max(1),
                (logical.height.round() as i32).max(1),
            );
        } else {
            #[allow(clippy::cast_possible_truncation)]
            let integer = (self.state.scale.round() as i32).max(1);
            self.state.layer().wl_surface().set_buffer_scale(integer);
        }
        self.state.layer().commit();
    }

    #[doc = include_str!("layer_window.doc/has_fractional_scale.md")]
    #[must_use]
    pub const fn has_fractional_scale(&self) -> bool {
        self.state.viewport.is_some()
    }

    #[doc = include_str!("layer_window.doc/target.md")]
    #[must_use]
    pub fn target(&self) -> LayerWindowTarget {
        let display = NonNull::new(self.connection.backend().display_ptr().cast())
            .expect("a live wayland display pointer");
        let surface = NonNull::new(self.state.layer().wl_surface().id().as_ptr().cast())
            .expect("a live wl_surface pointer");
        // SAFETY: both pointers name objects this `LayerWindow` owns, and the
        // target's contract is that it must not outlive it.
        unsafe { LayerWindowTarget::new(display, surface) }
    }
}

impl LayerBackend for LayerWindow {
    #[doc = include_str!("layer_window.doc/backend_open.md")]
    fn open(&mut self, settings: &LayerSettings) -> Result<(), LayerError> {
        // The connection and the loop belong to the surface that already exists,
        // so re-opening is applying: there is nothing here that a second
        // handshake would establish that this one has not.
        match self.apply(settings) {
            WindowOperationResult::Performed(_) => Ok(()),
            WindowOperationResult::Unsupported => Err(LayerError::Unsupported),
            WindowOperationResult::Failed(why) => Err(LayerError::BackendFailure(why)),
        }
    }

    fn apply(&mut self, settings: &LayerSettings) -> WindowOperationResult {
        if self.state.closed {
            return WindowOperationResult::failed("the surface has already been taken down");
        }
        // A display change is the one setting a live surface cannot carry: the
        // output is chosen when the surface is created and the protocol has no
        // request to move it. Asked before the new settings are stored, or the
        // comparison would be against themselves.
        let moved_display = settings.display.is_some() && settings.display != self.settings.display;

        apply(&self.compositor, self.state.layer(), settings);
        self.state.layer().commit();
        self.settings = settings.clone();

        if moved_display {
            WindowOperationResult::conditional()
        } else {
            WindowOperationResult::native()
        }
    }

    fn close(&mut self) -> WindowOperationResult {
        if self.state.closed {
            return WindowOperationResult::native();
        }
        self.state.closed = true;
        self.state
            .surface_events
            .push(LayerSurfaceEvent::CloseRequested);
        WindowOperationResult::native()
    }

    fn state(&self) -> LayerSurfaceState {
        if self.state.closed {
            LayerSurfaceState::Closed
        } else if self.state.configured {
            LayerSurfaceState::Ready
        } else {
            LayerSurfaceState::Configuring
        }
    }

    fn size(&self) -> Size {
        self.state.size
    }

    fn scale(&self) -> f32 {
        self.state.scale
    }

    #[doc = include_str!("layer_window.doc/supports.md")]
    fn supports(&self, feature: LayerFeature) -> SupportLevel {
        match feature {
            // wlr-layer-shell names all four bands, both anchor freedoms, all
            // three keyboard modes, a signed exclusive zone and a per-output
            // surface. Every one of these is the protocol's own vocabulary.
            LayerFeature::Bands
            | LayerFeature::Reservation
            | LayerFeature::FreeAnchor
            | LayerFeature::ExclusiveKeyboard
            | LayerFeature::DisplaySelection
            | LayerFeature::NegativeMargin
            // An empty input region is `wl_surface`'s own, not the layer
            // shell's, so it is there on every compositor that speaks Wayland
            // at all.
            | LayerFeature::InputTransparency => SupportLevel::Native,
            // Only when the compositor offered both halves of the fractional
            // path. Without them the integer path rounds up, which is a real
            // answer but not the one that was asked for.
            LayerFeature::FractionalScale => {
                if self.has_fractional_scale() {
                    SupportLevel::Native
                } else {
                    SupportLevel::Emulated
                }
            }
        }
    }

    fn poll_events(&mut self) -> Vec<LayerSurfaceEvent> {
        core::mem::take(&mut self.state.surface_events)
    }
}

#[doc = include_str!("layer_window.doc/apply.md")]
fn apply(compositor: &CompositorState, layer: &LayerSurface, settings: &LayerSettings) {
    layer.set_layer(WlrMapper::layer(settings.band));
    layer.set_anchor(WlrMapper::anchor(settings.anchor));
    layer.set_keyboard_interactivity(WlrMapper::keyboard(settings.focus));
    layer.set_exclusive_zone(WlrMapper::exclusive_zone(settings.reservation));
    layer.set_margin(
        settings.margin.top,
        settings.margin.right,
        settings.margin.bottom,
        settings.margin.left,
    );
    let (width, height) = WlrMapper::requested_size(settings);
    layer.set_size(width, height);
    apply_pointer(compositor, layer, settings.pointer);
}

#[doc = include_str!("layer_window.doc/apply_pointer.md")]
fn apply_pointer(compositor: &CompositorState, layer: &LayerSurface, policy: LayerPointerPolicy) {
    let surface = layer.wl_surface();
    if policy.takes_pointer() {
        // A null region is the protocol's own word for "all of it", and it is
        // what an untouched surface already has — set explicitly all the same,
        // because this path is also how a transparent surface becomes
        // interactive again on a reconfigure.
        surface.set_input_region(None);
        return;
    }
    match Region::new(compositor) {
        // Empty because nothing was added to it: input lands on whatever is
        // behind the surface instead. The region is copied into the pending
        // state here, so dropping it at the end of this function is correct.
        Ok(region) => surface.set_input_region(Some(region.wl_region())),
        Err(error) => eprintln!(
            "obscur-shell-layer: no wl_region here ({error}); \
             the surface stays clickable"
        ),
    }
}

#[doc = include_str!("layer_window.doc/initial_size.md")]
fn initial_size(settings: &LayerSettings) -> Size {
    #[allow(clippy::cast_precision_loss)]
    Size::new(
        settings.width.unwrap_or(0) as f32,
        settings.height.unwrap_or(0) as f32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_windowing::layer::LayerAnchor;

    #[test]
    fn a_bar_starts_at_the_thickness_it_asked_for() {
        // The stretched axis has no number yet — the compositor supplies it at
        // configure — and zero is what "not yet known" looks like in a `Size`.
        let size = initial_size(&LayerSettings::top_bar(36));
        assert_eq!(size.height, 36.0);
        assert_eq!(size.width, 0.0);
    }

    #[test]
    fn a_floating_surface_starts_at_its_explicit_size() {
        let settings = LayerSettings::new()
            .with_anchor(LayerAnchor::NONE)
            .with_size(320, 200);
        assert_eq!(initial_size(&settings), Size::new(320.0, 200.0));
    }
}
