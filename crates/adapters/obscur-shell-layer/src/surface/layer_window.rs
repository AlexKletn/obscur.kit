#![doc = include_str!("layer_window.doc/_module.md")]

use core::ptr::NonNull;

use calloop::EventLoop;
use calloop_wayland_source::WaylandSource;
use obscur_geometry::Size;
use obscur_windowing::layer::{
    LayerAnchor, LayerKeyboardInteractivity, LayerKind, LayerSurfaceConfig,
};
use smithay_client_toolkit::compositor::CompositorState;
use smithay_client_toolkit::output::OutputState;
use smithay_client_toolkit::registry::RegistryState;
use smithay_client_toolkit::seat::SeatState;
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::shell::wlr_layer::{
    Anchor, KeyboardInteractivity, Layer, LayerShell, LayerSurface,
};
use wayland_client::globals::registry_queue_init;
use wayland_client::{Connection, Proxy};
use wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1;
use wayland_protocols::wp::viewporter::client::wp_viewporter::WpViewporter;

use crate::error::LayerShellError;
use crate::event::LayerEvent;

use super::layer_state::LayerState;
use super::layer_waker::LayerWaker;
use super::layer_window_target::LayerWindowTarget;

#[doc = include_str!("layer_window.doc/LayerWindow.md")]
pub struct LayerWindow {
    connection: Connection,
    event_loop: EventLoop<'static, LayerState>,
    state: LayerState,
    #[doc = include_str!("layer_window.doc/_compositor.md")]
    _compositor: CompositorState,
    #[doc = include_str!("layer_window.doc/_fractional_manager.md")]
    _fractional_manager: Option<WpFractionalScaleManagerV1>,
    _viewporter: Option<WpViewporter>,
}

impl LayerWindow {
    #[doc = include_str!("layer_window.doc/open.md")]
    pub fn open(config: &LayerSurfaceConfig) -> Result<Self, LayerShellError> {
        Self::open_with(config, None)
    }

    #[doc = include_str!("layer_window.doc/open_with.md")]
    pub fn open_with(
        config: &LayerSurfaceConfig,
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
        // is what turns the registry's bare globals into named connectors.
        let mut state = LayerState::new(
            RegistryState::new(&globals),
            SeatState::new(&globals, &qh),
            OutputState::new(&globals, &qh),
            config.size,
        );
        let mut queue = queue;
        queue
            .roundtrip(&mut state)
            .map_err(|error| LayerShellError::Dispatch(error.to_string()))?;

        let chosen = config
            .output
            .name()
            .and_then(|name| state.output_named(name));

        let surface = compositor.create_surface(&qh);
        let layer = shell.create_layer_surface(
            &qh,
            surface,
            layer_of(config.layer),
            Some(config.namespace.clone()),
            chosen.as_ref(),
        );
        apply(&layer, config);

        // Fractional scaling, where the compositor offers it. Both halves are
        // needed: the fractional-scale object reports a scale in 120ths, and the
        // viewport is what maps a buffer rendered at that scale back onto the
        // logical size. With only one of the two the surface would be the wrong
        // size, so it is both or neither.
        let fractional_manager = globals
            .bind::<WpFractionalScaleManagerV1, _, _>(&qh, 1..=1, ())
            .ok();
        let viewporter = globals.bind::<WpViewporter, _, _>(&qh, 1..=1, ()).ok();
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
                    state.events.push(LayerEvent::Woken);
                })
                .map_err(|error| LayerShellError::Dispatch(error.to_string()))?;
        }

        Ok(Self {
            connection,
            event_loop,
            state,
            _compositor: compositor,
            _fractional_manager: fractional_manager,
            _viewporter: viewporter,
        })
    }

    #[doc = include_str!("layer_window.doc/dispatch.md")]
    pub fn dispatch(
        &mut self,
        timeout: Option<std::time::Duration>,
    ) -> Result<Vec<LayerEvent>, LayerShellError> {
        self.event_loop
            .dispatch(timeout, &mut self.state)
            .map_err(|error| LayerShellError::Dispatch(error.to_string()))?;
        Ok(core::mem::take(&mut self.state.events))
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

    #[doc = include_str!("layer_window.doc/size.md")]
    #[must_use]
    pub const fn size(&self) -> Size {
        self.state.size
    }

    #[doc = include_str!("layer_window.doc/scale.md")]
    #[must_use]
    pub const fn scale(&self) -> f32 {
        self.state.scale
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

    #[doc = include_str!("layer_window.doc/reconfigure.md")]
    pub fn reconfigure(&self, config: &LayerSurfaceConfig) {
        apply(self.state.layer(), config);
        self.state.layer().commit();
    }

    #[doc = include_str!("layer_window.doc/outputs.md")]
    #[must_use]
    pub fn outputs(&self) -> Vec<String> {
        self.state.output_names()
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

    #[doc = include_str!("layer_window.doc/physical_size.md")]
    #[must_use]
    pub fn physical_size(&self) -> Size {
        Size::new(
            (self.state.size.width * self.state.scale).round(),
            (self.state.size.height * self.state.scale).round(),
        )
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

#[doc = include_str!("layer_window.doc/apply.md")]
fn apply(layer: &LayerSurface, config: &LayerSurfaceConfig) {
    layer.set_layer(layer_of(config.layer));
    layer.set_anchor(anchor_of(config.anchor));
    layer.set_keyboard_interactivity(keyboard_of(config.keyboard_interactivity));
    layer.set_exclusive_zone(config.exclusive_zone.to_raw());
    layer.set_margin(
        config.margin.top,
        config.margin.right,
        config.margin.bottom,
        config.margin.left,
    );
    let (width, height) = config.requested_size();
    layer.set_size(width, height);
}

#[doc = include_str!("layer_window.doc/layer_of.md")]
const fn layer_of(kind: LayerKind) -> Layer {
    match kind {
        LayerKind::Background => Layer::Background,
        LayerKind::Bottom => Layer::Bottom,
        LayerKind::Top => Layer::Top,
        LayerKind::Overlay => Layer::Overlay,
    }
}

#[doc = include_str!("layer_window.doc/anchor_of.md")]
fn anchor_of(anchor: LayerAnchor) -> Anchor {
    let mut out = Anchor::empty();
    if anchor.top {
        out |= Anchor::TOP;
    }
    if anchor.right {
        out |= Anchor::RIGHT;
    }
    if anchor.bottom {
        out |= Anchor::BOTTOM;
    }
    if anchor.left {
        out |= Anchor::LEFT;
    }
    out
}

#[doc = include_str!("layer_window.doc/keyboard_of.md")]
const fn keyboard_of(interactivity: LayerKeyboardInteractivity) -> KeyboardInteractivity {
    match interactivity {
        LayerKeyboardInteractivity::None => KeyboardInteractivity::None,
        LayerKeyboardInteractivity::Exclusive => KeyboardInteractivity::Exclusive,
        LayerKeyboardInteractivity::OnDemand => KeyboardInteractivity::OnDemand,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The three enum bridges are pure mappings, and they are the one place a
    /// wrong pairing would put a panel in the wrong band with no error anywhere.
    #[test]
    fn every_band_maps_to_its_own_protocol_layer() {
        assert!(matches!(layer_of(LayerKind::Background), Layer::Background));
        assert!(matches!(layer_of(LayerKind::Bottom), Layer::Bottom));
        assert!(matches!(layer_of(LayerKind::Top), Layer::Top));
        assert!(matches!(layer_of(LayerKind::Overlay), Layer::Overlay));
    }

    #[test]
    fn a_top_bar_anchors_to_three_edges() {
        let anchor = anchor_of(LayerAnchor::TOP_BAR);
        assert!(anchor.contains(Anchor::TOP));
        assert!(anchor.contains(Anchor::LEFT));
        assert!(anchor.contains(Anchor::RIGHT));
        assert!(!anchor.contains(Anchor::BOTTOM));
    }

    #[test]
    fn an_empty_anchor_maps_to_no_bits() {
        assert!(anchor_of(LayerAnchor::NONE).is_empty());
        assert_eq!(anchor_of(LayerAnchor::ALL), Anchor::all());
    }

    #[test]
    fn keyboard_modes_map_one_to_one() {
        assert!(matches!(
            keyboard_of(LayerKeyboardInteractivity::None),
            KeyboardInteractivity::None
        ));
        assert!(matches!(
            keyboard_of(LayerKeyboardInteractivity::Exclusive),
            KeyboardInteractivity::Exclusive
        ));
        assert!(matches!(
            keyboard_of(LayerKeyboardInteractivity::OnDemand),
            KeyboardInteractivity::OnDemand
        ));
    }
}
