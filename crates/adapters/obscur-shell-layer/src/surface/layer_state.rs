#![doc = include_str!("layer_state.doc/_module.md")]

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
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};
use wayland_protocols::wp::fractional_scale::v1::client::{
    wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1,
    wp_fractional_scale_v1::{self, WpFractionalScaleV1},
};
use wayland_protocols::wp::viewporter::client::{
    wp_viewport::WpViewport, wp_viewporter::WpViewporter,
};

use crate::event::LayerEvent;

#[doc = include_str!("layer_state.doc/BTN_LEFT.md")]
const BTN_LEFT: u32 = 0x110;
#[doc = include_str!("layer_state.doc/BTN_RIGHT.md")]
const BTN_RIGHT: u32 = 0x111;
#[doc = include_str!("layer_state.doc/BTN_MIDDLE.md")]
const BTN_MIDDLE: u32 = 0x112;

#[doc = include_str!("layer_state.doc/LayerState.md")]
pub struct LayerState {
    pub(super) registry_state: RegistryState,
    pub(super) seat_state: SeatState,
    pub(super) output_state: OutputState,
    pub(super) layer: Option<LayerSurface>,
    #[doc = include_str!("layer_state.doc/events.md")]
    pub(super) events: Vec<LayerEvent>,
    #[doc = include_str!("layer_state.doc/size.md")]
    pub(super) size: Size,
    #[doc = include_str!("layer_state.doc/scale.md")]
    pub(super) scale: f32,
    #[doc = include_str!("layer_state.doc/closed.md")]
    pub(super) closed: bool,
    #[doc = include_str!("layer_state.doc/configured.md")]
    pub(super) configured: bool,
    pointer: Option<wl_pointer::WlPointer>,
    touch: Option<wl_touch::WlTouch>,
    #[doc = include_str!("layer_state.doc/touch_positions.md")]
    touch_positions: HashMap<i32, Point>,
    #[doc = include_str!("layer_state.doc/held.md")]
    held: PointerButtons,
    #[doc = include_str!("layer_state.doc/cursor.md")]
    cursor: Point,
    #[doc = include_str!("layer_state.doc/fractional.md")]
    pub(super) fractional: Option<WpFractionalScaleV1>,
    #[doc = include_str!("layer_state.doc/viewport.md")]
    pub(super) viewport: Option<WpViewport>,
}

impl LayerState {
    pub(super) fn new(
        registry_state: RegistryState,
        seat_state: SeatState,
        output_state: OutputState,
        size: Size,
    ) -> Self {
        Self {
            registry_state,
            seat_state,
            output_state,
            layer: None,
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
            fractional: None,
            viewport: None,
        }
    }

    #[doc = include_str!("layer_state.doc/layer.md")]
    pub(super) fn layer(&self) -> &LayerSurface {
        self.layer
            .as_ref()
            .expect("a layer surface, created before the window was handed out")
    }

    #[doc = include_str!("layer_state.doc/output_names.md")]
    pub(super) fn output_names(&self) -> Vec<String> {
        self.output_state
            .outputs()
            .filter_map(|output| self.output_state.info(&output))
            .filter_map(|info| info.name)
            .collect()
    }

    #[doc = include_str!("layer_state.doc/output_named.md")]
    pub(super) fn output_named(&self, name: &str) -> Option<wl_output::WlOutput> {
        self.output_state.outputs().find(|output| {
            self.output_state
                .info(output)
                .and_then(|info| info.name)
                .is_some_and(|connector| connector == name)
        })
    }

    #[doc = include_str!("layer_state.doc/push_pointer.md")]
    fn push_pointer(&mut self, event: PointerEvent) {
        self.events.push(LayerEvent::Pointer(event));
    }
}

#[doc = include_str!("layer_state.doc/button_of.md")]
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
        // A surface with a fractional-scale object gets its scale from there;
        // the integer event still arrives and would round 1.5 up to 2, undoing
        // the whole point of binding the protocol.
        if self.fractional.is_some() {
            return;
        }
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
                        self.events.push(LayerEvent::Scroll {
                            delta_y: delta * self.scale,
                            at: self.cursor,
                        });
                    }
                }
            }
        }
    }
}

#[doc = include_str!("layer_state.doc/WHEEL_NOTCH.md")]
const WHEEL_NOTCH: f32 = 48.0;

impl LayerState {
    #[doc = include_str!("layer_state.doc/push_event.md")]
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
        let at = point_of(position, self.scale);
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
        let live: Vec<(i32, Point)> = self.touch_positions.drain().collect();
        for (id, at) in live {
            self.push_touch(id, PointerEventKind::Cancel, at, 0, false);
        }
    }
}

#[doc = include_str!("layer_state.doc/point_of.md")]
fn point_of(position: (f64, f64), scale: f32) -> Point {
    #[allow(clippy::cast_possible_truncation)]
    Point::new(position.0 as f32 * scale, position.1 as f32 * scale)
}

impl LayerState {
    #[doc = include_str!("layer_state.doc/push_touch.md")]
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

#[doc = include_str!("layer_state.doc/FRACTIONAL_DENOMINATOR.md")]
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
                state.events.push(LayerEvent::ScaleChanged { scale });
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
