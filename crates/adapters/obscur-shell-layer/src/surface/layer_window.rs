//! [`LayerWindow`]: a live `wlr-layer-shell` surface and its dispatch loop.

use core::ptr::NonNull;

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
use wayland_client::{Connection, EventQueue, Proxy};
use wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1;
use wayland_protocols::wp::viewporter::client::wp_viewporter::WpViewporter;

use crate::error::LayerShellError;
use crate::event::LayerEvent;

use super::layer_state::LayerState;
use super::layer_window_target::LayerWindowTarget;

/// A live layer surface: the Wayland connection, the surface, and the queue that
/// drives them.
///
/// # Lifecycle
///
/// Opening does **not** give you a drawable surface. The protocol requires an
/// initial commit with no buffer, after which the compositor answers with a
/// configure carrying the real size — which is why
/// [`LayerEvent::Configured`](crate::event::LayerEvent::Configured) exists and
/// why [`is_configured`](Self::is_configured) gates the first frame. Drawing
/// before it is a protocol error, not a race.
///
/// # Driving it
///
/// ```ignore
/// let mut window = LayerWindow::open(&LayerSurfaceConfig::top_bar("panel", 32))?;
/// while !window.is_closed() {
///     for event in window.dispatch()? {
///         // feed pointer events to the runtime, resize on Configured, …
///     }
/// }
/// ```
///
/// [`dispatch`](Self::dispatch) blocks until the compositor says something;
/// [`dispatch_pending`](Self::dispatch_pending) does not, for a host that also
/// has its own timer or frame source.
pub struct LayerWindow {
    connection: Connection,
    queue: EventQueue<LayerState>,
    state: LayerState,
    /// The compositor global must outlive the surface made from it.
    _compositor: CompositorState,
    /// Kept alive for the same reason; absent where the compositor does not
    /// offer fractional scaling, which is what makes the integer path a
    /// fallback rather than a mode.
    _fractional_manager: Option<WpFractionalScaleManagerV1>,
    _viewporter: Option<WpViewporter>,
}

impl LayerWindow {
    /// Connects, binds `wlr-layer-shell`, and creates a surface described by
    /// `config`.
    ///
    /// # Errors
    ///
    /// [`LayerShellError::NoDisplay`] when there is no Wayland session,
    /// [`LayerShellError::Unsupported`] when the compositor does not implement
    /// the protocol — the expected answer on GNOME, and a capability result
    /// rather than a failure — and [`LayerShellError::MissingGlobal`] when a
    /// required global is absent.
    pub fn open(config: &LayerSurfaceConfig) -> Result<Self, LayerShellError> {
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

        Ok(Self {
            connection,
            queue,
            state,
            _compositor: compositor,
            _fractional_manager: fractional_manager,
            _viewporter: viewporter,
        })
    }

    /// Blocks until the compositor sends something, then returns everything that
    /// happened.
    ///
    /// # Errors
    ///
    /// [`LayerShellError::Dispatch`] when the connection fails.
    pub fn dispatch(&mut self) -> Result<Vec<LayerEvent>, LayerShellError> {
        self.queue
            .blocking_dispatch(&mut self.state)
            .map_err(|error| LayerShellError::Dispatch(error.to_string()))?;
        Ok(core::mem::take(&mut self.state.events))
    }

    /// Returns whatever has already arrived, without blocking.
    ///
    /// # Errors
    ///
    /// [`LayerShellError::Dispatch`] when the connection fails.
    pub fn dispatch_pending(&mut self) -> Result<Vec<LayerEvent>, LayerShellError> {
        self.queue
            .roundtrip(&mut self.state)
            .map_err(|error| LayerShellError::Dispatch(error.to_string()))?;
        Ok(core::mem::take(&mut self.state.events))
    }

    /// Flushes queued requests to the compositor.
    ///
    /// # Errors
    ///
    /// [`LayerShellError::Dispatch`] when the connection fails.
    pub fn flush(&self) -> Result<(), LayerShellError> {
        self.connection
            .flush()
            .map_err(|error| LayerShellError::Dispatch(error.to_string()))
    }

    /// The size the compositor last configured, in logical pixels.
    ///
    /// Before the first configure this is what was requested, which is a
    /// placeholder — [`is_configured`](Self::is_configured) says whether it is
    /// real yet.
    #[must_use]
    pub const fn size(&self) -> Size {
        self.state.size
    }

    /// The output scale the surface is on.
    #[must_use]
    pub const fn scale(&self) -> f32 {
        self.state.scale
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

    /// Re-applies `config` to the live surface — a panel changing its height, its
    /// anchor, or how much space it reserves.
    ///
    /// The compositor answers with a fresh configure; the host should treat the
    /// resulting [`LayerEvent::Configured`](crate::event::LayerEvent::Configured)
    /// as the new size rather than assuming the request was honoured verbatim.
    pub fn reconfigure(&self, config: &LayerSurfaceConfig) {
        apply(self.state.layer(), config);
        self.state.layer().commit();
    }

    /// Every output the compositor has announced, by connector name.
    ///
    /// A host reports this as its multi-output capability and offers it as the
    /// choice a user makes; it is the same set
    /// [`LayerOutput::Named`](obscur_windowing::layer::LayerOutput::Named)
    /// is matched against.
    #[must_use]
    pub fn outputs(&self) -> Vec<String> {
        self.state.output_names()
    }

    /// Tells the compositor how the buffer relates to logical pixels, by
    /// whichever mechanism this surface has.
    ///
    /// Without this a HiDPI surface is **twice the size it should be**: Wayland
    /// measures a surface in logical units as `buffer_size / buffer_scale`, so a
    /// buffer rendered at 2× and never declared as such is read as a surface
    /// twice as wide and twice as tall.
    ///
    /// Two mechanisms, and the caller should not have to know which is live:
    ///
    /// * **Fractional** — the buffer stays at scale `1` and a viewport maps it
    ///   onto the logical size. This is the only way to express 1.5, and it is
    ///   what avoids rendering a 1.6× display at 2× and letting the compositor
    ///   soften it back down.
    /// * **Integer** — `set_buffer_scale`, where fractional scaling is absent.
    ///   A fractional output then rounds up, as it always did.
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

    /// The buffer extent to render at, in physical pixels.
    ///
    /// This is the logical size times the surface's scale, rounded — the one
    /// place the two spaces are related, so a host never multiplies by hand.
    #[must_use]
    pub fn physical_size(&self) -> Size {
        Size::new(
            (self.state.size.width * self.state.scale).round(),
            (self.state.size.height * self.state.scale).round(),
        )
    }

    /// Whether the surface's scale can be fractional, i.e. whether the
    /// compositor offered `wp_fractional_scale_v1` **and** `wp_viewporter`.
    #[must_use]
    pub const fn has_fractional_scale(&self) -> bool {
        self.state.viewport.is_some()
    }

    /// The raw handles a `wgpu::Surface` is built from.
    ///
    /// # Panics
    ///
    /// If the Wayland library hands back a null display or surface pointer, which
    /// would mean the connection is already dead.
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

/// Pushes every field of `config` onto a live layer surface.
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

/// The protocol's band for a [`LayerKind`].
const fn layer_of(kind: LayerKind) -> Layer {
    match kind {
        LayerKind::Background => Layer::Background,
        LayerKind::Bottom => Layer::Bottom,
        LayerKind::Top => Layer::Top,
        LayerKind::Overlay => Layer::Overlay,
    }
}

/// The protocol's anchor bitset for a [`LayerAnchor`].
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

/// The protocol's keyboard mode for a [`LayerKeyboardInteractivity`].
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
