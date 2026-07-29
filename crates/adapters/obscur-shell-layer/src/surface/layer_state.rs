//! The Wayland dispatch state behind a layer surface.

use obscur_geometry::{Point, Size};
use obscur_interaction::pointer::{
    PointerButton, PointerButtons, PointerDevice, PointerEvent, PointerEventKind, PointerId,
};
use smithay_client_toolkit::compositor::CompositorHandler;
use smithay_client_toolkit::output::{OutputHandler, OutputState};
use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryState};
use smithay_client_toolkit::seat::pointer::{
    PointerEvent as SctkPointerEvent, PointerEventKind as SctkPointerKind, PointerHandler,
};
use smithay_client_toolkit::seat::touch::TouchHandler;
use smithay_client_toolkit::seat::{Capability, SeatHandler, SeatState};
use smithay_client_toolkit::shell::wlr_layer::{
    LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
};
use smithay_client_toolkit::{
    delegate_compositor, delegate_output, delegate_pointer, delegate_registry, delegate_seat,
    delegate_touch, registry_handlers,
};
use std::collections::HashMap;
use wayland_client::protocol::{wl_output, wl_pointer, wl_seat, wl_surface, wl_touch};
use wayland_client::{Connection, QueueHandle};

use crate::event::LayerEvent;

/// The left mouse button's Linux input code, which `wl_pointer` reports verbatim.
const BTN_LEFT: u32 = 0x110;
/// The right button.
const BTN_RIGHT: u32 = 0x111;
/// The middle button.
const BTN_MIDDLE: u32 = 0x112;

/// Everything the Wayland queue dispatches into, and the events it produces.
///
/// This is deliberately a plain data sink: the handlers translate Wayland into
/// the kit's own vocabulary and push onto [`events`](Self::events), and nothing
/// here draws, lays out or blocks. The host drains the queue after each
/// dispatch, so a slow frame cannot stall the connection.
pub(super) struct LayerState {
    pub(super) registry_state: RegistryState,
    pub(super) seat_state: SeatState,
    pub(super) output_state: OutputState,
    pub(super) layer: LayerSurface,
    /// Events produced since the host last drained.
    pub(super) events: Vec<LayerEvent>,
    /// The size the compositor last configured, in logical pixels.
    pub(super) size: Size,
    /// The output scale the surface is on.
    pub(super) scale: f32,
    /// Set once the compositor closes the surface.
    pub(super) closed: bool,
    /// Whether a configure has arrived, so the host knows when it may first draw.
    pub(super) configured: bool,
    pointer: Option<wl_pointer::WlPointer>,
    touch: Option<wl_touch::WlTouch>,
    /// The last position of each live touch contact, because `wl_touch.up`
    /// carries an id and a time but **no coordinates** — and an `Up` at the
    /// origin would read as a drag to the top-left corner.
    touch_positions: HashMap<i32, Point>,
    /// Buttons currently held, so a `Move` carries the same held set a drag needs.
    held: PointerButtons,
    /// The last pointer position, so a button press reports where it happened
    /// (`wl_pointer.button` carries no coordinates either).
    cursor: Point,
}

impl LayerState {
    pub(super) fn new(
        registry_state: RegistryState,
        seat_state: SeatState,
        output_state: OutputState,
        layer: LayerSurface,
        size: Size,
    ) -> Self {
        Self {
            registry_state,
            seat_state,
            output_state,
            layer,
            events: Vec::new(),
            size,
            scale: 1.0,
            closed: false,
            configured: false,
            pointer: None,
            touch: None,
            touch_positions: HashMap::new(),
            held: PointerButtons::new(),
            cursor: Point::new(0.0, 0.0),
        }
    }

    /// Pushes a converted pointer event, stamped with `time_ms` in microseconds.
    fn push_pointer(&mut self, event: PointerEvent) {
        self.events.push(LayerEvent::Pointer(event));
    }
}

/// The kit button for a Linux input code, or [`None`] for one the kit has no
/// name for — an extra thumb button is dropped rather than mapped onto a button
/// that means something else.
fn button_of(code: u32) -> Option<PointerButton> {
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
        _surface: &wl_surface::WlSurface,
        new_factor: i32,
    ) {
        #[allow(clippy::cast_precision_loss)]
        let scale = new_factor.max(1) as f32;
        self.scale = scale;
        self.events.push(LayerEvent::ScaleChanged { scale });
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
        self.closed = true;
        self.events.push(LayerEvent::Closed);
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        // A zero on an axis means the compositor left the choice to us; keep what
        // was asked for rather than collapsing the surface to nothing.
        let (width, height) = configure.new_size;
        #[allow(clippy::cast_precision_loss)]
        let size = Size::new(
            if width == 0 {
                self.size.width
            } else {
                width as f32
            },
            if height == 0 {
                self.size.height
            } else {
                height as f32
            },
        );
        self.size = size;
        self.configured = true;
        self.events.push(LayerEvent::Configured { size });
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
        match capability {
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
        _qh: &QueueHandle<Self>,
        _seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        match capability {
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

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl PointerHandler for LayerState {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[SctkPointerEvent],
    ) {
        for event in events {
            #[allow(clippy::cast_possible_truncation)]
            let at = Point::new(event.position.0 as f32, event.position.1 as f32);
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
                SctkPointerKind::Press { time, button, .. } => {
                    self.cursor = at;
                    if let Some(button) = button_of(button) {
                        self.held.press(button);
                    }
                    self.push_event(PointerEventKind::Down, at, time);
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
                        self.events.push(LayerEvent::Scroll { delta_y: delta });
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
        _touch: &wl_touch::WlTouch,
        _serial: u32,
        time: u32,
        _surface: wl_surface::WlSurface,
        id: i32,
        position: (f64, f64),
    ) {
        let at = point_of(position);
        self.touch_positions.insert(id, at);
        self.push_touch(id, PointerEventKind::Down, at, time, true);
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
        let at = point_of(position);
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
        let live: Vec<(i32, Point)> = self.touch_positions.drain().collect();
        for (id, at) in live {
            self.push_touch(id, PointerEventKind::Cancel, at, 0, false);
        }
    }
}

/// A surface-local Wayland position as a kit point.
fn point_of(position: (f64, f64)) -> Point {
    #[allow(clippy::cast_possible_truncation)]
    Point::new(position.0 as f32, position.1 as f32)
}

impl LayerState {
    /// Builds and pushes a touch contact's event.
    ///
    /// A contact holds the primary button while it is down, which is what makes
    /// the runtime treat a tap as a click without a touch-specific path.
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
delegate_pointer!(LayerState);
delegate_touch!(LayerState);
smithay_client_toolkit::delegate_layer!(LayerState);
