//! The Wayland dispatch state behind a layer surface.

use obscur_geometry::{Point, ScaleFactor, Size};
use obscur_interaction::pointer::{
    PointerButton, PointerButtons, PointerDevice, PointerEvent, PointerEventKind, PointerId,
};
use obscur_windowing::display::DisplayId;
use obscur_windowing::layer::{LayerDisplay, LayerPopupEvent, LayerSurfaceEvent};
use smithay_client_toolkit::compositor::CompositorHandler;
use smithay_client_toolkit::output::{OutputHandler, OutputState};
use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryState};
use smithay_client_toolkit::seat::pointer::{
    PointerEvent as SctkPointerEvent, PointerEventKind as SctkPointerKind, PointerHandler,
};
use smithay_client_toolkit::seat::touch::TouchHandler;
use smithay_client_toolkit::seat::{Capability, SeatHandler, SeatState};
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::shell::wlr_layer::{
    LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
};
use smithay_client_toolkit::{
    delegate_compositor, delegate_output, delegate_pointer, delegate_registry, delegate_seat,
    delegate_touch, registry_handlers,
};
use std::collections::HashMap;
use wayland_client::protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_surface, wl_touch};
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};
use wayland_protocols::wp::fractional_scale::v1::client::{
    wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1,
    wp_fractional_scale_v1::{self, WpFractionalScaleV1},
};
use wayland_protocols::wp::presentation_time::client::{
    wp_presentation::{self, WpPresentation},
    wp_presentation_feedback::{self, WpPresentationFeedback},
};
use wayland_protocols::wp::viewporter::client::{
    wp_viewport::WpViewport, wp_viewporter::WpViewporter,
};

use super::feedback_tag::FeedbackTag;
use crate::event::{Discarded, LayerInput, PopupActivation, Presented};

/// The left mouse button's Linux input code, which `wl_pointer` reports verbatim.
const BTN_LEFT: u32 = 0x110;
/// The right button.
const BTN_RIGHT: u32 = 0x111;
/// The middle button.
const BTN_MIDDLE: u32 = 0x112;

#[doc = include_str!("../../docs/api/surface/layer_state.doc/LayerState.md")]
pub struct LayerState {
    pub(super) registry_state: RegistryState,
    pub(super) seat_state: SeatState,
    pub(super) pointing_capabilities_dirty: bool,
    pub(super) output_state: OutputState,
    pub(super) popup: Option<super::layer_popup::LayerPopup>,
    pub(super) layer: Option<LayerSurface>,
    pub(super) popup_events: Vec<LayerPopupEvent>,
    pub(super) popup_shm: Option<smithay_client_toolkit::shm::Shm>,
    #[doc = include_str!("../../docs/api/surface/layer_state.doc/surface_events.md")]
    pub(super) surface_events: Vec<LayerSurfaceEvent>,
    #[doc = include_str!("../../docs/api/surface/layer_state.doc/input.md")]
    pub(super) input: Vec<(LayerInput, Option<PopupActivation>)>,
    /// The size the compositor last configured, in logical pixels.
    pub(super) size: Size,
    /// The latest application size to use on axes left to the client.
    pub(super) preferred_size: Size,
    /// The output scale the surface is on.
    pub(super) scale: f32,
    /// Set once the compositor closes the surface.
    pub(super) closed: bool,
    /// Whether a configure has arrived, so the host knows when it may first draw.
    pub(super) configured: bool,
    pointer: Option<wl_pointer::WlPointer>,
    pub(super) keyboard: Option<(wl_seat::WlSeat, wl_keyboard::WlKeyboard)>,
    pub(super) ime: super::layer_ime::LayerIme,
    pub(super) keyboard_target: Option<super::layer_keyboard::KeyboardTarget>,
    pub(super) consuming_menu_escape: bool,
    pub(super) seat_modifiers: obscur_interaction::command::Modifiers,
    pub(super) keyboard_loop: Option<calloop::LoopHandle<'static, Self>>,
    touch: Option<wl_touch::WlTouch>,
    /// The last position of each live touch contact, because `wl_touch.up`
    /// carries an id and a time but **no coordinates** — and an `Up` at the
    /// origin would read as a drag to the top-left corner.
    touch_positions: HashMap<i32, Point>,
    popup_touches: HashMap<i32, (wl_surface::WlSurface, Point)>,
    /// Buttons currently held, so a `Move` carries the same held set a drag needs.
    held: PointerButtons,
    /// The last pointer position, so a button press reports where it happened
    /// (`wl_pointer.button` carries no coordinates either).
    cursor: Point,
    /// The fractional-scale object for this surface, when the compositor offers
    /// the protocol. Its presence is what decides whether [`scale`](Self::scale)
    /// can be non-integral.
    pub(super) fractional: Option<WpFractionalScaleV1>,
    /// The viewport that maps the fractional buffer onto the logical size.
    pub(super) viewport: Option<WpViewport>,
    /// Bound when the compositor offers `wp_presentation`; absent when it does not,
    /// which costs the timings and nothing else.
    pub(super) presentation: Option<WpPresentation>,
    /// Frames the compositor has reported as shown, oldest first. Drained by
    /// `LayerWindow::take_presented`.
    pub(super) presented: Vec<Presented>,
    #[doc = include_str!("../../docs/api/surface/layer_state.doc/discarded.md")]
    pub(super) discarded: Vec<Discarded>,
    /// How many feedbacks have been asked for, which is what numbers them: a
    /// feedback binds to the next commit, so order is the only association there is.
    pub(super) feedbacks_asked: u64,
    #[doc = include_str!("../../docs/api/surface/layer_state.doc/commits.md")]
    pub(super) commits: std::cell::Cell<u64>,
    #[doc = include_str!("../../docs/api/surface/layer_state.doc/frame_callbacks.md")]
    pub(super) frame_callbacks: u64,
    /// Whether the compositor has said, since the last frame was asked for, that it is
    /// ready to composite this surface again.
    pub(super) frame_ready: bool,
}

impl LayerState {
    pub(super) fn push_input(&mut self, input: LayerInput) {
        self.input.push((input, None));
    }

    pub(super) fn mark_input_activation(&mut self, seat: &wl_seat::WlSeat, serial: u32) {
        if let Some(layer) = self.layer.as_ref()
            && let Some((_, activation)) = self.input.last_mut()
        {
            *activation = Some(PopupActivation {
                seat: seat.clone(),
                serial,
                parent: layer.wl_surface().clone(),
            });
        }
    }

    pub(super) fn new(
        registry_state: RegistryState,
        seat_state: SeatState,
        output_state: OutputState,
        size: Size,
        keyboard_loop: Option<calloop::LoopHandle<'static, Self>>,
    ) -> Self {
        Self {
            registry_state,
            seat_state,
            pointing_capabilities_dirty: true,
            output_state,
            layer: None,
            popup: None,
            popup_events: Vec::new(),
            popup_shm: None,
            surface_events: Vec::new(),
            input: Vec::new(),
            presentation: None,
            presented: Vec::new(),
            discarded: Vec::new(),
            feedbacks_asked: 0,
            commits: std::cell::Cell::new(0),
            frame_callbacks: 0,
            frame_ready: false,
            size,
            preferred_size: size,
            scale: 1.0,
            closed: false,
            configured: false,
            pointer: None,
            keyboard: None,
            ime: super::layer_ime::LayerIme::default(),
            keyboard_target: None,
            consuming_menu_escape: false,
            seat_modifiers: obscur_interaction::command::Modifiers::new(),
            keyboard_loop,
            touch: None,
            touch_positions: HashMap::new(),
            popup_touches: HashMap::new(),
            held: PointerButtons::new(),
            cursor: Point::new(0.0, 0.0),
            fractional: None,
            viewport: None,
        }
    }

    #[doc = include_str!("../../docs/api/surface/layer_state.doc/layer.md")]
    pub(super) fn layer(&self) -> &LayerSurface {
        self.layer
            .as_ref()
            .expect("a layer surface, created before the window was handed out")
    }

    #[doc = include_str!("../../docs/api/surface/layer_state.doc/commit.md")]
    pub(super) fn commit(&self) {
        self.layer().wl_surface().commit();
        self.commits.set(self.commits.get() + 1);
    }

    #[doc = include_str!("../../docs/api/surface/layer_state.doc/ask_when_drawable.md")]
    pub(super) fn ask_when_drawable(&mut self, qh: &QueueHandle<Self>) {
        let surface = self.layer().wl_surface().clone();
        // Bound to the surface's *next* commit, exactly as a presentation
        // feedback is, and asked in the same breath for the same reason: the
        // next commit is the frame. The compositor answers when it is ready to
        // composite this surface again — and answers **not at all** while it is
        // not compositing it, which is the one signal in the whole protocol that
        // says "a frame built now would not be seen".
        surface.frame(qh, surface.clone());
        self.frame_ready = false;
    }

    pub(super) fn mark_closed(&mut self) {
        if self.closed {
            return;
        }
        self.dismiss_popup();
        self.closed = true;
        self.surface_events.push(LayerSurfaceEvent::CloseRequested);
    }

    #[doc = include_str!("../../docs/api/surface/layer_state.doc/displays.md")]
    pub(super) fn displays(&self) -> Vec<LayerDisplay> {
        self.output_state
            .outputs()
            .filter_map(|output| {
                let info = self.output_state.info(&output)?;
                let id = Self::display_id(info.id);
                let name = info.name.unwrap_or_else(|| format!("output-{}", info.id));
                Some(LayerDisplay::new(id, name))
            })
            .collect()
    }

    #[doc = include_str!("../../docs/api/surface/layer_state.doc/output_with_id.md")]
    pub(super) fn output_with_id(&self, id: DisplayId) -> Option<wl_output::WlOutput> {
        self.output_state.outputs().find(|output| {
            self.output_state
                .info(output)
                .is_some_and(|info| Self::display_id(info.id) == id)
        })
    }

    #[doc = include_str!("../../docs/api/surface/layer_state.doc/display_id.md")]
    fn display_id(global_name: u32) -> DisplayId {
        // Keep zero reserved for callers that use it as an unset handle.
        DisplayId::from_raw(u64::from(global_name) + 1)
    }

    /// Pushes a converted pointer event, stamped with `time_ms` in microseconds.
    fn push_pointer(&mut self, event: PointerEvent) {
        self.push_input(LayerInput::Pointer(event));
    }
}

/// The kit button for a Linux input code, or [`None`] for one the kit has no
/// name for — an extra thumb button is dropped rather than mapped onto a button
/// that means something else.
pub(super) fn button_of(code: u32) -> Option<PointerButton> {
    match code {
        BTN_LEFT => Some(PointerButton::Primary),
        BTN_RIGHT => Some(PointerButton::Secondary),
        BTN_MIDDLE => Some(PointerButton::Middle),
        _ => None,
    }
}

impl CompositorHandler for LayerState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        new_factor: i32,
    ) {
        // A surface with a fractional-scale object gets its scale from there;
        // the integer event still arrives and would round 1.5 up to 2, undoing
        // the whole point of binding the protocol.
        if !self.is_panel_surface(surface) || self.fractional.is_some() {
            return;
        }
        #[allow(clippy::cast_precision_loss)]
        let scale = new_factor.max(1) as f32;
        self.scale = scale;
        self.surface_events
            .push(LayerSurfaceEvent::ScaleFactorChanged(ScaleFactor::new(
                scale,
            )));
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        // The compositor is about to composite this surface again, and this is
        // the only thing it ever says that means that. A surface it is not
        // compositing — hidden, animating from a snapshot, on another workspace
        // — gets no callback at all, so silence here is information too: it is
        // the difference between a frame that was slow and a frame that was
        // never going to be seen.
        self.frame_callbacks += 1;
        self.frame_ready = true;
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for LayerState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl LayerShellHandler for LayerState {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.mark_closed();
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if self.closed {
            return;
        }
        let (width, height) = configure.new_size;
        let size = Size::new(
            configured_axis(width, self.preferred_size.width, self.size.width),
            configured_axis(height, self.preferred_size.height, self.size.height),
        );
        // The first configure is what makes the surface drawable; every one
        // after it is the compositor changing the geometry underneath a surface
        // that was already up, and a host has reason to tell those apart.
        let event = if self.configured {
            LayerSurfaceEvent::Resized(size)
        } else {
            LayerSurfaceEvent::Configured(size)
        };
        self.size = size;
        self.configured = true;
        self.surface_events.push(event);
    }
}

#[allow(clippy::cast_precision_loss)]
fn configured_axis(suggested_px: u32, preferred_px: f32, previous_px: f32) -> f32 {
    match suggested_px {
        // Keep the last usable size when neither side gave an explicit extent.
        0 if preferred_px == 0.0 => previous_px,
        0 => preferred_px,
        suggested_px => suggested_px as f32,
    }
}

#[cfg(test)]
mod configure_tests {
    use super::configured_axis;

    #[test]
    fn zero_configure_uses_latest_preferred_extent() {
        assert_eq!(configured_axis(0, 32.0, 24.0), 32.0);
        assert_eq!(configured_axis(0, 0.0, 24.0), 24.0);
        assert_eq!(configured_axis(40, 32.0, 24.0), 40.0);
    }
}

impl SeatHandler for LayerState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        self.pointing_capabilities_dirty |=
            matches!(capability, Capability::Pointer | Capability::Touch);
        match capability {
            Capability::Keyboard => self.bind_keyboard(qh, seat),
            Capability::Pointer if self.pointer.is_none() => {
                self.pointer = self.seat_state.get_pointer(qh, &seat).ok();
            }
            Capability::Touch if self.touch.is_none() => {
                self.touch = self.seat_state.get_touch(qh, &seat).ok();
            }
            _ => {}
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        self.pointing_capabilities_dirty |=
            matches!(capability, Capability::Pointer | Capability::Touch);
        match capability {
            Capability::Keyboard => self.remove_keyboard(qh, &seat),
            Capability::Pointer => {
                if let Some(pointer) = self.pointer.take() {
                    pointer.release();
                }
            }
            Capability::Touch => {
                if let Some(touch) = self.touch.take() {
                    touch.release();
                }
            }
            _ => {}
        }
    }

    fn remove_seat(&mut self, _: &Connection, qh: &QueueHandle<Self>, seat: wl_seat::WlSeat) {
        self.pointing_capabilities_dirty = true;
        self.remove_keyboard(qh, &seat);
    }
}

impl PointerHandler for LayerState {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        pointer: &wl_pointer::WlPointer,
        events: &[SctkPointerEvent],
    ) {
        for event in events {
            if !self.is_panel_surface(&event.surface) {
                self.popup_pointer(event);
                continue;
            }
            let at = point_of(event.position, self.scale);
            match event.kind {
                SctkPointerKind::Enter { .. } => {
                    self.cursor = at;
                    self.push_event(PointerEventKind::Enter, at, 0);
                }
                SctkPointerKind::Leave { .. } => {
                    self.push_event(PointerEventKind::Leave, at, 0);
                }
                SctkPointerKind::Motion { time } => {
                    self.cursor = at;
                    self.push_event(PointerEventKind::Move, at, time);
                }
                SctkPointerKind::Press {
                    time,
                    button,
                    serial,
                } => {
                    if self.popup.as_ref().is_some_and(|popup| popup.grabbed) {
                        self.dismiss_popup();
                    }
                    self.cursor = at;
                    if let Some(button) = button_of(button) {
                        self.held.press(button);
                    }
                    self.push_event(PointerEventKind::Down, at, time);
                    if let Some(data) =
                        pointer.data::<smithay_client_toolkit::seat::pointer::PointerData>()
                    {
                        self.mark_input_activation(data.seat(), serial);
                    }
                }
                SctkPointerKind::Release { time, button, .. } => {
                    self.cursor = at;
                    if let Some(button) = button_of(button) {
                        self.held.release(button);
                    }
                    self.push_event(PointerEventKind::Up, at, time);
                }
                SctkPointerKind::Axis { vertical, .. } => {
                    // `absolute` is in logical pixels already; `discrete` counts
                    // notches, which a compositor may send instead.
                    #[allow(clippy::cast_possible_truncation)]
                    let delta = if vertical.absolute == 0.0 {
                        #[allow(clippy::cast_precision_loss)]
                        {
                            f32::from(vertical.discrete as i16) * WHEEL_NOTCH
                        }
                    } else {
                        vertical.absolute as f32
                    };
                    if delta != 0.0 {
                        self.push_input(LayerInput::Scroll {
                            delta_y: delta * self.scale,
                            at: self.cursor,
                        });
                    }
                }
            }
        }
    }
}

/// Logical pixels a discrete wheel notch scrolls, matching the windowed runner.
const WHEEL_NOTCH: f32 = 48.0;

impl LayerState {
    /// Builds and pushes a mouse pointer event at `at`, stamped from `time_ms`.
    fn push_event(&mut self, kind: PointerEventKind, at: Point, time_ms: u32) {
        let event = PointerEvent::new(PointerId::MOUSE, PointerDevice::Mouse, kind, at)
            .with_buttons(self.held)
            .with_timestamp(u64::from(time_ms) * 1_000)
            .as_trusted();
        self.push_pointer(event);
    }
}

impl TouchHandler for LayerState {
    fn down(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        touch: &wl_touch::WlTouch,
        serial: u32,
        time: u32,
        surface: wl_surface::WlSurface,
        id: i32,
        position: (f64, f64),
    ) {
        if !self.is_panel_surface(&surface) {
            let at = point_of(position, 1.0);
            self.popup_touch(&surface, id, PointerEventKind::Down, at, time);
            self.popup_touches.insert(id, (surface, at));
            return;
        }
        if self.popup.as_ref().is_some_and(|popup| popup.grabbed) {
            self.dismiss_popup();
        }
        let at = point_of(position, self.scale);
        self.touch_positions.insert(id, at);
        self.push_touch(id, PointerEventKind::Down, at, time, true);
        if let Some(data) = touch.data::<smithay_client_toolkit::seat::touch::TouchData>() {
            self.mark_input_activation(data.seat(), serial);
        }
    }

    fn up(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _touch: &wl_touch::WlTouch,
        _serial: u32,
        time: u32,
        id: i32,
    ) {
        if let Some((surface, at)) = self.popup_touches.remove(&id) {
            self.popup_touch(&surface, id, PointerEventKind::Up, at, time);
            return;
        }
        // No coordinates on `up`: use the contact's last known position, or the
        // origin only if it never moved anywhere (which cannot happen after a
        // `down`).
        let at = self
            .touch_positions
            .remove(&id)
            .unwrap_or_else(|| Point::new(0.0, 0.0));
        self.push_touch(id, PointerEventKind::Up, at, time, false);
    }

    fn motion(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _touch: &wl_touch::WlTouch,
        time: u32,
        id: i32,
        position: (f64, f64),
    ) {
        if let Some((surface, last)) = self.popup_touches.get_mut(&id) {
            let at = point_of(position, 1.0);
            *last = at;
            let surface = surface.clone();
            self.popup_touch(&surface, id, PointerEventKind::Move, at, time);
            return;
        }
        let at = point_of(position, self.scale);
        self.touch_positions.insert(id, at);
        self.push_touch(id, PointerEventKind::Move, at, time, true);
    }

    fn shape(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _touch: &wl_touch::WlTouch,
        _id: i32,
        _major: f64,
        _minor: f64,
    ) {
    }

    fn orientation(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _touch: &wl_touch::WlTouch,
        _id: i32,
        _orientation: f64,
    ) {
    }

    fn cancel(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _touch: &wl_touch::WlTouch) {
        // Every live contact is gone at once, and each must be told — a contact
        // left believing it is down would hold a capture forever.
        let popup_touches = core::mem::take(&mut self.popup_touches);
        for (id, (surface, at)) in popup_touches {
            self.popup_touch(&surface, id, PointerEventKind::Cancel, at, 0);
        }
        let live: Vec<(i32, Point)> = self.touch_positions.drain().collect();
        for (id, at) in live {
            self.push_touch(id, PointerEventKind::Cancel, at, 0, false);
        }
    }
}

#[doc = include_str!("../../docs/api/surface/layer_state.doc/point_of.md")]
fn point_of(position: (f64, f64), scale: f32) -> Point {
    #[allow(clippy::cast_possible_truncation)]
    Point::new(position.0 as f32 * scale, position.1 as f32 * scale)
}

impl LayerState {
    #[doc = include_str!("../../docs/api/surface/layer_state.doc/push_touch.md")]
    fn push_touch(&mut self, id: i32, kind: PointerEventKind, at: Point, time_ms: u32, down: bool) {
        let mut held = PointerButtons::new();
        if down {
            held.press(PointerButton::Primary);
        }
        #[allow(clippy::cast_sign_loss)]
        let pointer = PointerId::touch(id as u64);
        let event = PointerEvent::new(pointer, PointerDevice::Touch, kind, at)
            .with_buttons(held)
            .with_timestamp(u64::from(time_ms) * 1_000)
            .as_trusted();
        self.push_pointer(event);
    }
}

/// The denominator `wp_fractional_scale_v1` reports its scale in: the event
/// carries `scale × 120`, so 1.5 arrives as 180.
const FRACTIONAL_DENOMINATOR: f32 = 120.0;

impl Dispatch<WpFractionalScaleV1, ()> for LayerState {
    fn event(
        state: &mut Self,
        _proxy: &WpFractionalScaleV1,
        event: wp_fractional_scale_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let wp_fractional_scale_v1::Event::PreferredScale { scale } = event {
            #[allow(clippy::cast_precision_loss)]
            let scale = scale as f32 / FRACTIONAL_DENOMINATOR;
            if scale > 0.0 && (scale - state.scale).abs() > f32::EPSILON {
                state.scale = scale;
                state
                    .surface_events
                    .push(LayerSurfaceEvent::ScaleFactorChanged(ScaleFactor::new(
                        scale,
                    )));
            }
        }
    }
}

impl Dispatch<WpFractionalScaleManagerV1, ()> for LayerState {
    fn event(
        _state: &mut Self,
        _proxy: &WpFractionalScaleManagerV1,
        _event: <WpFractionalScaleManagerV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // The manager has no events.
    }
}

impl Dispatch<WpPresentation, ()> for LayerState {
    fn event(
        _state: &mut Self,
        _proxy: &WpPresentation,
        event: wp_presentation::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // `clock_id` is the one event it has, and it says which clock the
        // timestamps below are in. Every Linux compositor answers
        // `CLOCK_MONOTONIC`, which is the clock input events already carry, so
        // nothing here converts anything — the two are directly comparable.
        // A compositor that answered something else would make the numbers
        // wrong rather than absent, so it is worth saying so out loud.
        if let wp_presentation::Event::ClockId { clk_id } = event
            && clk_id != 1
        {
            eprintln!(
                "obscur-shell-layer: the compositor times presentation on clock {clk_id}, \
                 not CLOCK_MONOTONIC; presentation times cannot be compared with input timestamps"
            );
        }
    }
}

impl Dispatch<WpPresentationFeedback, FeedbackTag> for LayerState {
    fn event(
        state: &mut Self,
        _proxy: &WpPresentationFeedback,
        event: wp_presentation_feedback::Event,
        tag: &FeedbackTag,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            wp_presentation_feedback::Event::Presented {
                tv_sec_hi,
                tv_sec_lo,
                tv_nsec,
                refresh,
                seq_hi,
                seq_lo,
                flags,
            } => {
                state.presented.push(Presented {
                    at_micros: obscur_wayland_effects::presentation_timestamp_micros(
                        tv_sec_hi, tv_sec_lo, tv_nsec,
                    ),
                    refresh_nanos: refresh,
                    seq: tag.seq,
                    // The screen's own counter, which used to go into `..` along
                    // with the flags. Without it, "the panel never turned over"
                    // and "we never handed it anything" are the same reading.
                    msc: (u64::from(seq_hi) << 32) | u64::from(seq_lo),
                    flags: flags.into(),
                });
            }
            // The frame was never shown — superseded by a later commit, or the
            // surface was not being composited. Recorded rather than printed
            // from here: the runner prints it next to the presentations, on the
            // same stream and in the same order, and the two causes are only
            // told apart by the commit count that travels with it.
            wp_presentation_feedback::Event::Discarded => {
                state.discarded.push(Discarded {
                    seq: tag.seq,
                    commits_since_ask: state.commits.get().saturating_sub(tag.commits_at_ask),
                });
            }
            _ => {}
        }
    }
}

impl Dispatch<WpViewporter, ()> for LayerState {
    fn event(
        _state: &mut Self,
        _proxy: &WpViewporter,
        _event: <WpViewporter as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // The viewporter has no events.
    }
}

impl Dispatch<WpViewport, ()> for LayerState {
    fn event(
        _state: &mut Self,
        _proxy: &WpViewport,
        _event: <WpViewport as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // A viewport has no events.
    }
}

impl ProvidesRegistryState for LayerState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

delegate_registry!(LayerState);
delegate_compositor!(LayerState);
delegate_output!(LayerState);
delegate_seat!(LayerState);
smithay_client_toolkit::delegate_keyboard!(LayerState);
delegate_pointer!(LayerState);
delegate_touch!(LayerState);
smithay_client_toolkit::delegate_layer!(LayerState);

#[cfg(test)]
mod tests {
    use super::*;

    /// The conversion that decides whether a click lands where the finger did.
    /// Wayland speaks logical surface coordinates; the runtime divides by the
    /// device scale, so the adapter must hand it physical ones.
    #[test]
    fn a_wayland_position_is_scaled_into_physical_pixels() {
        assert_eq!(point_of((100.0, 20.0), 2.0), Point::new(200.0, 40.0));
        assert_eq!(point_of((100.0, 20.0), 1.0), Point::new(100.0, 20.0));
    }

    #[test]
    fn only_the_three_named_buttons_map_to_a_kit_button() {
        assert_eq!(button_of(BTN_LEFT), Some(PointerButton::Primary));
        assert_eq!(button_of(BTN_RIGHT), Some(PointerButton::Secondary));
        assert_eq!(button_of(BTN_MIDDLE), Some(PointerButton::Middle));
        // An extra thumb button is dropped rather than mapped onto one that
        // already means something else.
        assert_eq!(button_of(0x116), None);
    }
}
