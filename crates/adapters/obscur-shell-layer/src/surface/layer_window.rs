//! [`LayerWindow`]: a live `wlr-layer-shell` surface and its dispatch loop.

use calloop::EventLoop;
use calloop_wayland_source::WaylandSource;
use obscur_capability::SupportLevel;
use obscur_geometry::{Rect, Size};
use obscur_windowing::layer::{
    LayerDisplay, LayerEdge, LayerFocusPolicy, LayerPointerPolicy, LayerPopupEvent,
    LayerPopupFrame, LayerSettings, LayerSurfaceEvent, LayerSurfaceState,
};
use obscur_windowing::port::{LayerBackend, LayerError, LayerFeature};
use obscur_windowing::request::WindowOperationResult;
use smithay_client_toolkit::compositor::{CompositorState, Region};
use smithay_client_toolkit::output::OutputState;
use smithay_client_toolkit::registry::RegistryState;
use smithay_client_toolkit::seat::SeatState;
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::shell::wlr_layer::{LayerShell, LayerSurface, SurfaceKind};
use wayland_client::globals::registry_queue_init;
use wayland_client::{Connection, Proxy};
use wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1;
use wayland_protocols::wp::presentation_time::client::wp_presentation::WpPresentation;
use wayland_protocols::wp::viewporter::client::wp_viewporter::WpViewporter;

use crate::error::LayerShellError;
use crate::event::{Discarded, LayerInput, PopupActivation, Presented};

use super::feedback_tag::FeedbackTag;
use super::layer_state::LayerState;
use super::layer_waker::LayerWaker;
use super::layer_window_target::LayerWindowTarget;
use super::wlr_mapper::WlrMapper;

#[doc = include_str!("../../docs/api/surface/layer_window.doc/LayerWindow.md")]
pub struct LayerWindow {
    background_blur: Option<obscur_wayland_effects::BackgroundBlur>,
    connection: Connection,
    event_loop: EventLoop<'static, LayerState>,
    state: LayerState,
    #[doc = include_str!("../../docs/api/surface/layer_window.doc/settings.md")]
    settings: LayerSettings,
    /// The compositor global must outlive the surface made from it — and is asked for
    /// a `wl_region` whenever the pointer policy changes.
    compositor: CompositorState,
    popup_shell: Option<smithay_client_toolkit::shell::xdg::XdgShell>,
    /// Kept alive for the same reason; absent where the compositor does not
    /// offer fractional scaling, which is what makes the integer path a
    /// fallback rather than a mode.
    _fractional_manager: Option<WpFractionalScaleManagerV1>,
    _viewporter: Option<WpViewporter>,
    /// The Wayland queue handle, kept because a request made after `open` — a
    /// presentation feedback, for one — needs one and the loop only carries calloop.
    queue: wayland_client::QueueHandle<LayerState>,
}

impl LayerWindow {
    #[doc = include_str!("../../docs/api/surface/layer_window.doc/open.md")]
    pub fn open(settings: &LayerSettings, namespace: &str) -> Result<Self, LayerShellError> {
        Self::open_with(settings, namespace, None)
    }

    #[doc = include_str!("../../docs/api/surface/layer_window.doc/open_with.md")]
    pub fn open_with(
        settings: &LayerSettings,
        namespace: &str,
        waker: Option<&LayerWaker>,
    ) -> Result<Self, LayerShellError> {
        let connection = Connection::connect_to_env().map_err(LayerShellError::NoDisplay)?;
        let (globals, queue) =
            registry_queue_init::<LayerState>(&connection).map_err(LayerShellError::Registry)?;
        let qh = queue.handle();

        let compositor = CompositorState::bind(&globals, &qh)
            .map_err(|_| LayerShellError::MissingGlobal("wl_compositor".to_owned()))?;
        let shell = LayerShell::bind(&globals, &qh).map_err(|_| LayerShellError::Unsupported)?;

        // Outputs must be enumerated before the surface can name one, so the
        // state exists first and the surface is created into it. One roundtrip
        // is what turns the registry's bare globals into identified displays.
        let event_loop: EventLoop<'static, LayerState> =
            EventLoop::try_new().map_err(LayerShellError::EventLoop)?;
        let mut state = LayerState::new(
            RegistryState::new(&globals),
            SeatState::new(&globals, &qh),
            OutputState::new(&globals, &qh),
            preferred_size(settings),
            Some(event_loop.handle()),
        );
        state.ime.manager = globals.bind::<
            wayland_protocols::wp::text_input::zv3::client::zwp_text_input_manager_v3::ZwpTextInputManagerV3,
            _, _,
        >(&qh, 1..=1, ()).ok();
        let popup_shell = smithay_client_toolkit::shell::xdg::XdgShell::bind(&globals, &qh).ok();
        state.popup_shm = smithay_client_toolkit::shm::Shm::bind(&globals, &qh).ok();
        let mut queue = queue;
        queue
            .roundtrip(&mut state)
            .map_err(LayerShellError::Roundtrip)?;

        let chosen = settings
            .display
            .map(|id| {
                state.output_with_id(id).ok_or_else(|| {
                    LayerShellError::InvalidSettings(format!(
                        "requested display {} is unavailable",
                        id.raw()
                    ))
                })
            })
            .transpose()?;

        let surface = compositor.create_surface(&qh);
        let layer = shell.create_layer_surface(
            &qh,
            surface,
            WlrMapper::layer(settings.band),
            Some(namespace.to_owned()),
            chosen.as_ref(),
        );
        if settings.focus == LayerFocusPolicy::OnDemand && layer_version(&layer) < 4 {
            return Err(LayerShellError::Unsupported);
        }
        apply(&compositor, &layer, settings).map_err(LayerShellError::Dispatch)?;

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
        state.commits.set(state.commits.get() + 1);

        WaylandSource::new(connection.clone(), queue)
            .insert(event_loop.handle())
            .map_err(|error| LayerShellError::RegisterSource(error.error))?;
        if let Some(waker) = waker {
            waker.register_source(|source| {
                event_loop
                    .handle()
                    .insert_source(source, |(), (), state: &mut LayerState| {
                        // The turn alone is not enough: a host renders when it has a
                        // reason to, so the wake has to become one.
                        state.surface_events.push(LayerSurfaceEvent::Woken);
                    })
            })?;
        }

        Ok(Self {
            background_blur: None,
            connection,
            event_loop,
            state,
            settings: settings.clone(),
            compositor,
            popup_shell,
            _fractional_manager: fractional_manager,
            _viewporter: viewporter,
            queue: qh,
        })
    }

    /// Stages background blur for the next buffer commit.
    ///
    /// Returns whether the compositor offers a blur protocol for the surface.
    ///
    /// # Errors
    ///
    /// Returns [`LayerShellError::Dispatch`] if Wayland blur setup fails.
    pub fn request_background_blur(&mut self) -> Result<bool, LayerShellError> {
        self.set_background_blur(true)
    }

    /// Stages a blur update for the next surface commit.
    ///
    /// Returns whether the surface has a Wayland blur object.
    ///
    /// # Errors
    ///
    /// Returns [`LayerShellError::Dispatch`] if enabling blur requires a new
    /// Wayland blur object and its setup fails.
    pub fn set_background_blur(&mut self, enabled: bool) -> Result<bool, LayerShellError> {
        if let Some(effect) = self.background_blur.as_mut() {
            effect.set_enabled(enabled);
        } else if enabled {
            self.background_blur =
                obscur_wayland_effects::BackgroundBlur::request(self.state.layer().wl_surface())
                    .map_err(LayerShellError::Dispatch)?;
        }
        Ok(self.background_blur.is_some())
    }

    #[doc = include_str!("../../docs/api/surface/layer_window.doc/dispatch.md")]
    pub fn dispatch(
        &mut self,
        timeout: Option<std::time::Duration>,
    ) -> Result<(), LayerShellError> {
        self.event_loop
            .dispatch(timeout, &mut self.state)
            .map_err(LayerShellError::EventLoop)
    }

    /// Synchronize the writable panel field with Wayland text-input v3.
    pub fn set_text_input(
        &mut self,
        field: Option<(obscur_identity::NodeId, &str, usize, usize, Option<Rect>)>,
    ) {
        let field = field.filter(|_| {
            self.state.keyboard_route() == Some(super::layer_keyboard::KeyboardTarget::Panel)
        });
        self.state.ime.sync(field);
    }

    #[doc = include_str!("../../docs/api/surface/layer_window.doc/poll_input.md")]
    pub fn poll_input(&mut self) -> Vec<LayerInput> {
        self.poll_input_with_activation()
            .into_iter()
            .map(|(input, _)| input)
            .collect()
    }

    /// Drain panel input with an optional activation for button, key and touch presses.
    pub fn poll_input_with_activation(&mut self) -> Vec<(LayerInput, Option<PopupActivation>)> {
        core::mem::take(&mut self.state.input)
    }

    /// Open a popup with the Wayland grab authorized by the initiating input event.
    ///
    /// # Errors
    ///
    /// Returns [`LayerError::Unsupported`] when popup support is unavailable,
    /// [`LayerError::InvalidSettings`] when the panel, anchor or activation
    /// cannot be used, and [`LayerError::BackendError`] when Wayland popup
    /// setup fails.
    pub fn show_grabbed_popup(
        &mut self,
        anchor: Rect,
        side: LayerEdge,
        frame: LayerPopupFrame,
        activation: PopupActivation,
    ) -> Result<(), LayerError> {
        if !self.state.is_panel_surface(&activation.parent) || !activation.seat.is_alive() {
            return Err(LayerError::InvalidSettings(
                "popup activation does not belong to this live panel".into(),
            ));
        }
        self.open_popup(anchor, side, frame, Some(activation))
    }

    fn open_popup(
        &mut self,
        anchor: Rect,
        side: LayerEdge,
        frame: LayerPopupFrame,
        activation: Option<PopupActivation>,
    ) -> Result<(), LayerError> {
        let (Some(shell), Some(shm)) = (&self.popup_shell, &self.state.popup_shm) else {
            return Err(LayerError::Unsupported);
        };
        if !self.is_configured() || self.is_closed() {
            return Err(LayerError::InvalidSettings("the panel is not ready".into()));
        }
        let mut popup = super::layer_popup::LayerPopup::new(
            anchor,
            side,
            frame,
            self.state.size,
            &self.compositor,
            shell,
            shm,
            &self.queue,
        )?;
        self.state.dismiss_popup();
        popup.attach(self.state.layer(), activation);
        self.state.popup = Some(popup);
        Ok(())
    }

    #[doc = include_str!("../../docs/api/surface/layer_window.doc/ask_when_shown.md")]
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
        //
        // The commit count is taken now so that the answer, whenever it comes,
        // can say whether this process committed over its own frame in between.
        // A `discarded` with no commits since the ask is the compositor's doing;
        // one with commits since the ask is ours.
        presentation.feedback(
            layer.wl_surface(),
            &self.queue,
            FeedbackTag {
                seq,
                commits_at_ask: self.state.commits.get(),
            },
        );
    }

    #[doc = include_str!("../../docs/api/surface/layer_window.doc/ask_when_drawable.md")]
    pub fn ask_when_drawable(&mut self) {
        if self.state.layer.is_none() {
            return;
        }
        let queue = self.queue.clone();
        self.state.ask_when_drawable(&queue);
    }

    #[doc = include_str!("../../docs/api/surface/layer_window.doc/is_drawable_now.md")]
    #[must_use]
    pub const fn is_drawable_now(&self) -> bool {
        self.state.frame_ready
    }

    #[doc = include_str!("../../docs/api/surface/layer_window.doc/frame_callbacks.md")]
    #[must_use]
    pub const fn frame_callbacks(&self) -> u64 {
        self.state.frame_callbacks
    }

    /// Takes the presentations reported since the last call, oldest first.
    pub fn take_presented(&mut self) -> Vec<Presented> {
        core::mem::take(&mut self.state.presented)
    }

    #[doc = include_str!("../../docs/api/surface/layer_window.doc/take_discarded.md")]
    pub fn take_discarded(&mut self) -> Vec<Discarded> {
        core::mem::take(&mut self.state.discarded)
    }

    #[doc = include_str!("../../docs/api/surface/layer_window.doc/add_turn_waker.md")]
    pub fn add_turn_waker(&self, waker: &LayerWaker) -> Result<(), LayerShellError> {
        waker.register_source(|source| {
            self.event_loop
                .handle()
                .insert_source(source, |(), (), _state: &mut LayerState| {
                    // Deliberately empty. Ringing this means "your `dispatch` may
                    // return now", not "something changed" — the caller looks at
                    // whatever it was waiting for. Pushing a `Woken` here would tell
                    // the host to reconsider the interface, which is a different
                    // claim and would cost a frame nobody asked for.
                })
        })
    }

    #[doc = include_str!("../../docs/api/surface/layer_window.doc/loop_handle.md")]
    #[must_use]
    pub fn loop_handle(&self) -> calloop::LoopHandle<'static, LayerState> {
        self.event_loop.handle()
    }

    #[doc = include_str!("../../docs/api/surface/layer_window.doc/flush.md")]
    pub fn flush(&self) -> Result<(), LayerShellError> {
        self.connection.flush().map_err(LayerShellError::Flush)
    }

    /// Whether the first configure has arrived, so the surface may be drawn to.
    #[must_use]
    pub const fn is_configured(&self) -> bool {
        self.state.configured
    }

    /// Whether the compositor has closed the surface.
    #[must_use]
    pub const fn is_closed(&self) -> bool {
        self.state.closed
    }

    /// The settings this surface is currently living under.
    #[must_use]
    pub const fn settings(&self) -> &LayerSettings {
        &self.settings
    }

    #[doc = include_str!("../../docs/api/surface/layer_window.doc/displays.md")]
    #[must_use]
    pub fn displays(&self) -> Vec<LayerDisplay> {
        self.state.displays()
    }

    #[doc = include_str!("../../docs/api/surface/layer_window.doc/sync_scale.md")]
    pub fn sync_scale(&self) -> Result<(), LayerShellError> {
        use super::protocol_coordinate::protocol_coordinate;

        let logical = self.state.size;
        if let Some(viewport) = self.state.viewport.as_ref() {
            let width = protocol_coordinate(logical.width.round()).ok_or_else(|| {
                LayerShellError::InvalidSettings(
                    "viewport width exceeds the Wayland coordinate range".into(),
                )
            })?;
            let height = protocol_coordinate(logical.height.round()).ok_or_else(|| {
                LayerShellError::InvalidSettings(
                    "viewport height exceeds the Wayland coordinate range".into(),
                )
            })?;
            self.state.layer().wl_surface().set_buffer_scale(1);
            viewport.set_destination(width.max(1), height.max(1));
        } else {
            let integer = protocol_coordinate(self.state.scale.round()).ok_or_else(|| {
                LayerShellError::InvalidSettings(
                    "buffer scale exceeds the Wayland coordinate range".into(),
                )
            })?;
            self.state
                .layer()
                .wl_surface()
                .set_buffer_scale(integer.max(1));
        }
        self.state.commit();
        Ok(())
    }

    /// Whether the surface's scale can be fractional, i.e. whether the
    /// compositor offered `wp_fractional_scale_v1` **and** `wp_viewporter`.
    #[must_use]
    pub const fn has_fractional_scale(&self) -> bool {
        self.state.viewport.is_some()
    }

    /// Returns the initial pointing capabilities or a new aggregate snapshot after a Wayland seat update.
    pub fn take_pointing_capabilities(&mut self) -> Option<super::LayerPointingCapabilities> {
        if !std::mem::take(&mut self.state.pointing_capabilities_dirty) {
            return None;
        }
        let seats = &self.state.seat_state;
        let mut capabilities = super::LayerPointingCapabilities::default();
        for info in seats.seats().filter_map(|seat| seats.info(&seat)) {
            capabilities.pointer |= info.has_pointer;
            capabilities.touch |= info.has_touch;
        }
        Some(capabilities)
    }

    /// Returns a renderer target that shares ownership of the Wayland connection
    /// and layer surface. The renderer may retain the target after this window is
    /// dropped. The window owns event dispatch and geometry updates.
    #[must_use]
    pub fn target(&self) -> LayerWindowTarget {
        LayerWindowTarget::new(self.connection.clone(), self.state.layer().clone())
    }
}

impl LayerBackend for LayerWindow {
    #[doc = include_str!("../../docs/api/surface/layer_window.doc/backend_open.md")]
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
        // Layer-shell binds the output when it creates the surface.
        if settings.display != self.settings.display {
            return WindowOperationResult::failed(
                "changing the display requires reopening the layer surface",
            );
        }

        // Settings that are the settings the surface already has are not a
        // reconfiguration, and this is the difference between saying so and not:
        // every one of these ends in a bare `wl_surface.commit`, and a bare
        // commit landing on a surface whose last frame the compositor has not
        // shown yet supersedes that frame — the content update is replaced by
        // one carrying no new buffer, and the presentation feedback bound to it
        // comes back `discarded`. An application that answers each keystroke by
        // re-stating the geometry it already asked for therefore throws away the
        // frame it just paid to build, once per keystroke.
        if *settings == self.settings {
            return WindowOperationResult::native();
        }

        if settings.focus == LayerFocusPolicy::OnDemand && layer_version(self.state.layer()) < 4 {
            return WindowOperationResult::failed(
                "on-demand keyboard focus requires layer-shell v4",
            );
        }
        if let Err(reason) = apply(&self.compositor, self.state.layer(), settings) {
            return WindowOperationResult::failed(reason);
        }
        self.state.preferred_size = preferred_size(settings);
        self.state.commit();
        self.settings = settings.clone();

        WindowOperationResult::native()
    }

    fn close(&mut self) -> WindowOperationResult {
        self.state.mark_closed();
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

    #[doc = include_str!("../../docs/api/surface/layer_window.doc/supports.md")]
    fn supports(&self, feature: LayerFeature) -> SupportLevel {
        super::layer_probe::layer_support(
            feature,
            self.has_fractional_scale(),
            layer_version(self.state.layer()),
            self.popup_shell.is_some() && self.state.popup_shm.is_some(),
            self.state.ime.manager.is_some(),
        )
    }

    fn show_popup(
        &mut self,
        anchor: Rect,
        side: LayerEdge,
        frame: LayerPopupFrame,
    ) -> Result<(), LayerError> {
        self.open_popup(anchor, side, frame, None)
    }

    fn update_popup(&mut self, frame: LayerPopupFrame) -> Result<(), LayerError> {
        if self.supports(LayerFeature::Popup) == SupportLevel::Unavailable {
            return Err(LayerError::Unsupported);
        }
        self.state
            .popup
            .as_mut()
            .ok_or_else(|| LayerError::InvalidSettings("no popup is open".into()))?
            .update(frame)
    }

    fn close_popup(&mut self) -> WindowOperationResult {
        if self.supports(LayerFeature::Popup) == SupportLevel::Unavailable {
            return WindowOperationResult::Unsupported;
        }
        self.state.dismiss_popup();
        WindowOperationResult::native()
    }

    fn poll_popup_events(&mut self) -> Vec<LayerPopupEvent> {
        core::mem::take(&mut self.state.popup_events)
    }

    fn poll_events(&mut self) -> Vec<LayerSurfaceEvent> {
        core::mem::take(&mut self.state.surface_events)
    }
}

fn layer_version(layer: &LayerSurface) -> u32 {
    match layer.kind() {
        SurfaceKind::Wlr(surface) => surface.version(),
        _ => 0,
    }
}

#[doc = include_str!("../../docs/api/surface/layer_window.doc/apply.md")]
fn apply(
    compositor: &CompositorState,
    layer: &LayerSurface,
    settings: &LayerSettings,
) -> Result<(), String> {
    // Creating the input region can fail. Do it before queuing any other settings.
    apply_pointer(compositor, layer, settings.pointer)?;
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
    Ok(())
}

#[doc = include_str!("../../docs/api/surface/layer_window.doc/apply_pointer.md")]
fn apply_pointer(
    compositor: &CompositorState,
    layer: &LayerSurface,
    policy: LayerPointerPolicy,
) -> Result<(), String> {
    let surface = layer.wl_surface();
    if policy.takes_pointer() {
        // A null region is the protocol's own word for "all of it", and it is
        // what an untouched surface already has — set explicitly all the same,
        // because this path is also how a transparent surface becomes
        // interactive again on a reconfigure.
        surface.set_input_region(None);
        return Ok(());
    }
    let region = Region::new(compositor)
        .map_err(|error| format!("could not create input region: {error}"))?;
    // SCTK can return an inert proxy when the Wayland constructor fails.
    if !region.wl_region().is_alive() {
        return Err("Wayland returned an inert input region".into());
    }
    // Wayland copies the region into pending state, so the proxy can be dropped.
    surface.set_input_region(Some(region.wl_region()));
    Ok(())
}

#[doc = include_str!("../../docs/api/surface/layer_window.doc/preferred_size.md")]
fn preferred_size(settings: &LayerSettings) -> Size {
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
        let size = preferred_size(&LayerSettings::top_bar(36));
        assert_eq!(size.height, 36.0);
        assert_eq!(size.width, 0.0);
    }

    #[test]
    fn a_floating_surface_starts_at_its_explicit_size() {
        let settings = LayerSettings::new()
            .with_anchor(LayerAnchor::NONE)
            .with_size(320, 200);
        assert_eq!(preferred_size(&settings), Size::new(320.0, 200.0));
    }

    #[test]
    #[ignore = "requires a live Wayland compositor with layer-shell"]
    fn opening_on_an_unavailable_display_reports_invalid_settings() {
        use obscur_windowing::display::DisplayId;

        let settings = LayerSettings::top_bar(24).with_display(DisplayId::from_raw(0));
        assert!(matches!(
            LayerWindow::open(&settings, "obscur-layer-display-test"),
            Err(LayerShellError::InvalidSettings(reason))
                if reason == "requested display 0 is unavailable"
        ));
    }

    #[test]
    #[ignore = "requires a live Wayland compositor with layer-shell"]
    fn changing_the_display_rejects_the_whole_reconfiguration() {
        use crate::surface::LayerProbe;

        let display = LayerProbe::displays().unwrap().remove(0);
        let settings = LayerSettings::top_bar(24).with_display(display.id());
        let mut window = LayerWindow::open(&settings, "obscur-layer-settings-test").unwrap();
        let requested = LayerSettings::top_bar(32);

        for _ in 0..2 {
            let result = window.apply(&requested);
            assert_eq!(
                result.failure_reason(),
                Some("changing the display requires reopening the layer surface")
            );
            assert_eq!(window.settings(), &settings);
        }
        let transparent = requested
            .with_display(display.id())
            .with_pointer(LayerPointerPolicy::Transparent);
        assert!(window.apply(&transparent).is_success());
        assert_eq!(window.settings(), &transparent);
    }

    #[test]
    #[ignore = "requires a live Wayland compositor with layer-shell"]
    fn native_popup_configures_and_closes_on_a_live_panel() {
        use obscur_geometry::Point;
        use std::time::{Duration, Instant};

        let settings = LayerSettings::new().with_size(32, 24);
        let mut window = LayerWindow::open(&settings, "obscur-popup-smoke").unwrap();
        assert_eq!(window.supports(LayerFeature::Popup), SupportLevel::Native);
        let deadline = Instant::now() + Duration::from_secs(3);
        while !window.is_configured() && Instant::now() < deadline {
            window.dispatch(Some(Duration::from_millis(25))).unwrap();
        }
        assert!(window.is_configured());
        if window.has_fractional_scale() {
            let size = window.state.size;
            let commits = window.state.commits.get();
            window.state.size.width = 2_147_483_648.0;
            assert!(matches!(
                window.sync_scale(),
                Err(LayerShellError::InvalidSettings(_))
            ));
            assert_eq!(window.state.commits.get(), commits);
            window.state.size = size;
        }
        window.sync_scale().unwrap();
        let pixels = LayerPopupFrame::new(32, 24, vec![0; 32 * 24 * 4]).unwrap();
        let buffer = super::super::layer_popup::popup_buffer(
            &pixels,
            window.state.popup_shm.as_ref().unwrap(),
        )
        .unwrap();
        buffer.attach_to(window.state.layer().wl_surface()).unwrap();
        window.state.commit();

        let anchor = Rect::new(Point::new(4.0, 4.0), Size::new(8.0, 8.0));
        for _ in 0..2 {
            let frame = LayerPopupFrame::new(24, 12, vec![0; 24 * 12 * 4]).unwrap();
            window.show_popup(anchor, LayerEdge::Bottom, frame).unwrap();
            let deadline = Instant::now() + Duration::from_secs(3);
            let mut configured = None;
            while configured.is_none() && Instant::now() < deadline {
                window.dispatch(Some(Duration::from_millis(25))).unwrap();
                for event in window.poll_popup_events() {
                    match event {
                        LayerPopupEvent::Configured(rect) => configured = Some(rect),
                        LayerPopupEvent::Failed(reason) => panic!("{reason}"),
                        _ => {}
                    }
                }
            }
            assert_eq!(configured.unwrap().size, Size::new(24.0, 12.0));
            let frame = LayerPopupFrame::new(24, 12, vec![80; 24 * 12 * 4]).unwrap();
            let surface = window.state.popup.as_ref().unwrap().surface().id();
            window.update_popup(frame).unwrap();
            window.dispatch(Some(Duration::from_millis(25))).unwrap();
            assert_eq!(window.state.popup.as_ref().unwrap().surface().id(), surface);
            assert!(window.poll_popup_events().iter().all(|event| {
                !matches!(event, LayerPopupEvent::Closed | LayerPopupEvent::Failed(_))
            }));
        }
        assert!(window.close().is_success());
        assert!(window.close().is_success());
        assert_eq!(window.state(), LayerSurfaceState::Closed);
        assert_eq!(
            window
                .poll_events()
                .iter()
                .filter(|event| matches!(event, LayerSurfaceEvent::CloseRequested))
                .count(),
            1
        );
        assert!(matches!(
            window.poll_popup_events().as_slice(),
            [LayerPopupEvent::Closed]
        ));
        window.dispatch(Some(Duration::from_millis(25))).unwrap();
    }
    #[test]
    #[ignore = "injects input only into an explicitly selected isolated compositor"]
    fn native_grabbed_popup_dismisses_on_outside_press_and_escape() {
        use obscur_geometry::Point;
        use obscur_interaction::pointer::PointerEventKind;
        use obscur_windowing::layer::LayerReservation;
        use std::io::Write;
        use std::os::fd::AsFd;
        use std::time::{Duration, Instant};
        use wayland_client::protocol::wl_pointer::ButtonState;
        use wayland_client::protocol::wl_seat::WlSeat;
        use wayland_protocols_misc::zwp_input_method_v2::client::{
            zwp_input_method_manager_v2::ZwpInputMethodManagerV2,
            zwp_input_method_v2::{self, ZwpInputMethodV2},
        };
        use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
            zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1,
            zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
        };
        use wayland_protocols_wlr::virtual_pointer::v1::client::{
            zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1,
            zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1,
        };
        use xkbcommon::xkb;

        #[derive(Default)]
        struct VirtualInput {
            ime_serial: u32,
            activated: bool,
        }
        impl Dispatch<ZwpInputMethodV2, ()> for VirtualInput {
            fn event(
                state: &mut Self,
                _: &ZwpInputMethodV2,
                event: zwp_input_method_v2::Event,
                _: &(),
                _: &Connection,
                _: &QueueHandle<Self>,
            ) {
                match event {
                    zwp_input_method_v2::Event::Activate => state.activated = true,
                    zwp_input_method_v2::Event::Deactivate => state.activated = false,
                    zwp_input_method_v2::Event::Done => state.ime_serial += 1,
                    zwp_input_method_v2::Event::Unavailable => {
                        panic!("test input method unavailable")
                    }
                    _ => {}
                }
            }
        }
        use wayland_client::{
            Dispatch, QueueHandle, globals::GlobalListContents, protocol::wl_registry,
        };
        impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for VirtualInput {
            fn event(
                _: &mut Self,
                _: &wl_registry::WlRegistry,
                _: wl_registry::Event,
                _: &GlobalListContents,
                _: &Connection,
                _: &QueueHandle<Self>,
            ) {
            }
        }

        wayland_client::delegate_noop!(VirtualInput: ignore ZwlrVirtualPointerManagerV1);
        wayland_client::delegate_noop!(VirtualInput: ignore ZwlrVirtualPointerV1);
        wayland_client::delegate_noop!(VirtualInput: ignore ZwpVirtualKeyboardManagerV1);
        wayland_client::delegate_noop!(VirtualInput: ignore ZwpVirtualKeyboardV1);
        wayland_client::delegate_noop!(VirtualInput: ignore WlSeat);
        wayland_client::delegate_noop!(VirtualInput: ignore ZwpInputMethodManagerV2);

        let socket = std::env::var("OBSCUR_ISOLATED_POPUP_TEST_SOCKET").unwrap();
        assert_eq!(std::env::var("WAYLAND_DISPLAY").unwrap(), socket);
        assert!(socket.starts_with("/tmp/okp."));
        let connection = Connection::connect_to_env().unwrap();
        let (globals, mut event_queue) = registry_queue_init::<VirtualInput>(&connection).unwrap();
        let queue = event_queue.handle();
        let manager = globals
            .bind::<ZwlrVirtualPointerManagerV1, _, _>(&queue, 1..=2, ())
            .unwrap();
        let pointer = manager.create_virtual_pointer(None, &queue, ());

        let seat = globals.bind::<WlSeat, _, _>(&queue, 1..=1, ()).unwrap();
        let keyboard_manager = globals
            .bind::<ZwpVirtualKeyboardManagerV1, _, _>(&queue, 1..=1, ())
            .unwrap();
        let keyboard = keyboard_manager.create_virtual_keyboard(&seat, &queue, ());
        let ime_manager = globals
            .bind::<ZwpInputMethodManagerV2, _, _>(&queue, 1..=1, ())
            .unwrap();
        let ime = ime_manager.get_input_method(&seat, &queue, ());
        let mut virtual_input = VirtualInput::default();
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let keymap = xkb::Keymap::new_from_names(
            &context,
            "",
            "",
            "us",
            "",
            None,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        )
        .unwrap();
        let mut keymap_text = keymap.get_as_string(xkb::KEYMAP_FORMAT_TEXT_V1);
        keymap_text.push('\0');
        let keymap_path = std::path::Path::new(&socket).with_extension("test-keymap");
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&keymap_path)
            .unwrap();
        file.write_all(keymap_text.as_bytes()).unwrap();
        std::fs::remove_file(keymap_path).unwrap();
        keyboard.keymap(1, file.as_fd(), u32::try_from(keymap_text.len()).unwrap());

        let settings = LayerSettings::new()
            .with_anchor(LayerAnchor {
                top: true,
                left: true,
                ..LayerAnchor::NONE
            })
            .with_size(240, 48)
            .with_focus(LayerFocusPolicy::OnDemand)
            .with_reservation(LayerReservation::None);
        let mut window = LayerWindow::open(&settings, "obscur-menu-smoke").unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while !window.is_configured() && Instant::now() < deadline {
            window.dispatch(Some(Duration::from_millis(25))).unwrap();
        }
        assert!(window.is_configured());
        let width = window.size().width as u32;
        let parent = LayerPopupFrame::new(width, 48, vec![0; width as usize * 48 * 4]).unwrap();
        let buffer = super::super::layer_popup::popup_buffer(
            &parent,
            window.state.popup_shm.as_ref().unwrap(),
        )
        .unwrap();
        buffer.attach_to(window.state.layer().wl_surface()).unwrap();
        window.state.commit();
        window.dispatch(Some(Duration::from_millis(50))).unwrap();

        let open_menu = |window: &mut LayerWindow, time: u32| {
            pointer.motion_absolute(time, 10, 10, 1000, 1000);
            pointer.frame();
            pointer.button(time + 1, 0x110, ButtonState::Pressed);
            pointer.frame();
            connection.flush().unwrap();

            let deadline = Instant::now() + Duration::from_secs(3);
            let mut activation = None;
            while activation.is_none() && Instant::now() < deadline {
                window.dispatch(Some(Duration::from_millis(25))).unwrap();
                activation = window.poll_input_with_activation().into_iter().find_map(|(event, activation)| {
                matches!(event, LayerInput::Pointer(pointer) if pointer.kind() == PointerEventKind::Down)
                    .then_some(activation).flatten()
            });
            }
            let frame = LayerPopupFrame::new(32, 24, vec![0; 32 * 24 * 4]).unwrap();
            let anchor = Rect::new(Point::new(width as f32 * 0.1, 4.0), Size::new(8.0, 8.0));
            window
                .show_grabbed_popup(
                    anchor,
                    LayerEdge::Bottom,
                    frame,
                    activation.expect("native panel press"),
                )
                .unwrap();

            let deadline = Instant::now() + Duration::from_secs(3);
            let mut configured = false;
            let mut focused = false;
            while !(configured && focused) && Instant::now() < deadline {
                window.dispatch(Some(Duration::from_millis(25))).unwrap();
                for event in window.poll_popup_events() {
                    assert!(
                        !matches!(event, LayerPopupEvent::Closed | LayerPopupEvent::Failed(_)),
                        "{event:?}"
                    );
                    configured |= matches!(event, LayerPopupEvent::Configured(_));
                    focused |= matches!(event, LayerPopupEvent::KeyboardFocus(true));
                }
            }
            assert!(
                configured && focused,
                "configured={configured}, keyboard_focus={focused}"
            );
        };
        open_menu(&mut window, 1);
        pointer.button(3, 0x110, ButtonState::Released);
        pointer.frame();
        pointer.motion_absolute(4, 900, 900, 1000, 1000);
        pointer.frame();
        pointer.button(5, 0x110, ButtonState::Pressed);
        pointer.frame();
        pointer.button(6, 0x110, ButtonState::Released);
        pointer.frame();
        connection.flush().unwrap();

        let await_close = |window: &mut LayerWindow| {
            let deadline = Instant::now() + Duration::from_secs(3);
            let mut closed = false;
            while !closed && Instant::now() < deadline {
                window.dispatch(Some(Duration::from_millis(25))).unwrap();
                closed = window
                    .poll_popup_events()
                    .into_iter()
                    .any(|event| matches!(event, LayerPopupEvent::Closed));
            }
            assert!(closed);
        };
        await_close(&mut window);

        open_menu(&mut window, 10);
        pointer.button(12, 0x110, ButtonState::Released);
        pointer.frame();
        keyboard.key(13, 1, 1);
        keyboard.key(14, 1, 0);
        connection.flush().unwrap();
        await_close(&mut window);

        assert_eq!(
            window.supports(LayerFeature::TextInput),
            SupportLevel::Native
        );
        let node = obscur_identity::NodeId::from_raw(42);
        let deadline = Instant::now() + Duration::from_secs(3);
        while !(virtual_input.activated && virtual_input.ime_serial > 0)
            && Instant::now() < deadline
        {
            window.dispatch(Some(Duration::from_millis(25))).unwrap();
            window.set_text_input(Some((
                node,
                "é",
                2,
                2,
                Some(Rect::new(Point::new(4.0, 4.0), Size::new(8.0, 8.0))),
            )));
            event_queue.roundtrip(&mut virtual_input).unwrap();
        }
        assert!(virtual_input.activated);
        ime.set_preedit_string("に".into(), 3, 3);
        ime.commit(virtual_input.ime_serial);
        connection.flush().unwrap();

        let poll_update = |window: &mut LayerWindow| {
            let deadline = Instant::now() + Duration::from_secs(3);
            while Instant::now() < deadline {
                window.dispatch(Some(Duration::from_millis(25))).unwrap();
                if let Some(update) =
                    window
                        .poll_input()
                        .into_iter()
                        .find_map(|event| match event {
                            LayerInput::TextInput(target, update) if target == node => Some(update),
                            _ => None,
                        })
                {
                    return update;
                }
            }
            panic!("no native text input update");
        };
        let update = poll_update(&mut window);
        assert_eq!(update.preedit.unwrap().text(), "に");
        ime.commit_string("日".into());
        ime.commit(virtual_input.ime_serial);
        connection.flush().unwrap();
        assert_eq!(poll_update(&mut window).commit.as_deref(), Some("日"));
    }
}
