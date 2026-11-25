use std::collections::HashMap;
use std::num::NonZeroUsize;

use obscur_geometry::Point;
use obscur_interaction::event::EventTrust;
use obscur_interaction::gesture::{GesturePhase, TouchpadGestureDelta, TouchpadGestureEvent};
use wayland_client::backend::ObjectId;
use wayland_client::globals::GlobalListContents;
use wayland_client::protocol::{wl_pointer, wl_registry, wl_seat};
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle, WEnum};
use wayland_protocols::wp::pointer_gestures::zv1::client::{
    zwp_pointer_gesture_pinch_v1 as pinch, zwp_pointer_gesture_swipe_v1 as swipe,
    zwp_pointer_gestures_v1 as gestures,
};

use super::{gesture_pointer::GesturePointer, gesture_seat::GestureSeat};

pub(super) struct GestureState {
    surface: ObjectId,
    manager: Option<gestures::ZwpPointerGesturesV1>,
    seats: HashMap<u32, GestureSeat>,
    events: Vec<TouchpadGestureEvent>,
    active: Option<(u32, TouchpadGestureEvent)>,
}

impl GestureState {
    pub(super) fn new(surface: ObjectId, manager: Option<gestures::ZwpPointerGesturesV1>) -> Self {
        Self {
            surface,
            manager,
            seats: HashMap::new(),
            events: Vec::new(),
            active: None,
        }
    }

    pub(super) fn take_events(&mut self) -> Vec<TouchpadGestureEvent> {
        std::mem::take(&mut self.events)
    }

    pub(super) fn pointing_capabilities(&self) -> Option<crate::WaylandPointingCapabilities> {
        self.seats.values().try_fold(
            crate::WaylandPointingCapabilities::default(),
            |mut aggregate, seat| {
                let info = seat.capabilities?;
                aggregate.pointer |= info.pointer;
                aggregate.touch |= info.touch;
                Some(aggregate)
            },
        )
    }

    pub(super) fn add_seat(
        &mut self,
        registry: &wl_registry::WlRegistry,
        queue: &QueueHandle<Self>,
        name: u32,
        version: u32,
    ) {
        self.seats.entry(name).or_insert_with(|| GestureSeat {
            proxy: registry.bind(name, version.min(7), queue, name),
            pointer: None,
            capabilities: None,
        });
    }

    fn begin(&mut self, name: u32, surface: ObjectId, fingers: u32, delta: TouchpadGestureDelta) {
        if surface != self.surface || self.active.is_some() {
            return;
        }
        if let Some(pointer) = self.seats.get(&name).and_then(|seat| seat.pointer.as_ref()) {
            let event = TouchpadGestureEvent {
                phase: GesturePhase::Started,
                position: pointer.position,
                finger_count: NonZeroUsize::new(fingers as usize),
                delta,
                trust: EventTrust::Trusted,
                timestamp_micros: 0,
            };
            self.active = Some((name, event));
            self.events.push(event);
        }
    }

    fn update(&mut self, name: u32, delta: TouchpadGestureDelta) {
        let Some((_, event)) = self.active.as_mut().filter(|(seat, _)| *seat == name) else {
            return;
        };
        event.phase = GesturePhase::Updated;
        event.delta = match (event.delta, delta) {
            (
                TouchpadGestureDelta::Transform {
                    rotation: previous, ..
                },
                TouchpadGestureDelta::Transform { scale, rotation },
            ) => TouchpadGestureDelta::Transform {
                scale,
                rotation: previous + rotation,
            },
            (_, delta) => delta,
        };
        self.events.push(*event);
    }

    fn finish(&mut self, name: u32, cancelled: bool) {
        if self.active.is_some_and(|(seat, _)| seat == name)
            && let Some((_, mut event)) = self.active.take()
        {
            event.phase = if cancelled {
                GesturePhase::Cancelled
            } else {
                GesturePhase::Completed
            };
            if matches!(event.delta, TouchpadGestureDelta::Swipe(_)) {
                event.delta = TouchpadGestureDelta::Swipe(Point::new(0.0, 0.0));
            }
            self.events.push(event);
        }
    }
}

impl Drop for GestureState {
    fn drop(&mut self) {
        self.seats.clear();
        if let Some(manager) = &self.manager
            && manager.version() >= 2
        {
            manager.release();
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for GestureState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        queue: &QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } if interface == wl_seat::WlSeat::interface().name => {
                state.add_seat(registry, queue, name, version);
            }
            wl_registry::Event::GlobalRemove { name } => {
                state.finish(name, true);
                state.seats.remove(&name);
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_seat::WlSeat, u32> for GestureState {
    fn event(
        state: &mut Self,
        proxy: &wl_seat::WlSeat,
        event: wl_seat::Event,
        name: &u32,
        _: &Connection,
        queue: &QueueHandle<Self>,
    ) {
        let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
        } = event
        else {
            return;
        };
        let Some(seat) = state.seats.get_mut(name) else {
            return;
        };
        let pointer_available = capabilities.contains(wl_seat::Capability::Pointer);
        seat.capabilities = Some(crate::WaylandPointingCapabilities {
            pointer: pointer_available,
            touch: capabilities.contains(wl_seat::Capability::Touch),
        });
        if !pointer_available {
            seat.pointer = None;
            state.finish(*name, true);
            return;
        }
        if let Some(manager) = &state.manager {
            seat.pointer.get_or_insert_with(|| {
                let proxy = proxy.get_pointer(queue, *name);
                let swipe = manager.get_swipe_gesture(&proxy, queue, *name);
                let pinch = manager.get_pinch_gesture(&proxy, queue, *name);
                GesturePointer {
                    proxy,
                    swipe,
                    pinch,
                    position: Point::new(0.0, 0.0),
                }
            });
        }
    }
}

impl Dispatch<wl_pointer::WlPointer, u32> for GestureState {
    fn event(
        state: &mut Self,
        proxy: &wl_pointer::WlPointer,
        event: wl_pointer::Event,
        name: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if matches!(event, wl_pointer::Event::Leave { .. }) {
            state.finish(*name, true);
        }
        if let Some(pointer) = state
            .seats
            .get_mut(name)
            .and_then(|seat| seat.pointer.as_mut())
            .filter(|pointer| pointer.proxy == *proxy)
        {
            match event {
                wl_pointer::Event::Enter {
                    surface_x,
                    surface_y,
                    ..
                }
                | wl_pointer::Event::Motion {
                    surface_x,
                    surface_y,
                    ..
                } => {
                    pointer.position = Point::new(surface_x as f32, surface_y as f32);
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<swipe::ZwpPointerGestureSwipeV1, u32> for GestureState {
    fn event(
        state: &mut Self,
        _: &swipe::ZwpPointerGestureSwipeV1,
        event: swipe::Event,
        name: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let zero = Point::new(0.0, 0.0);
        match event {
            swipe::Event::Begin {
                surface, fingers, ..
            } => state.begin(
                *name,
                surface.id(),
                fingers,
                TouchpadGestureDelta::Swipe(zero),
            ),
            swipe::Event::Update { dx, dy, .. } => state.update(
                *name,
                TouchpadGestureDelta::Swipe(Point::new(dx as f32, dy as f32)),
            ),
            swipe::Event::End { cancelled, .. } => state.finish(*name, cancelled != 0),
            _ => {}
        }
    }
}

impl Dispatch<pinch::ZwpPointerGesturePinchV1, u32> for GestureState {
    fn event(
        state: &mut Self,
        _: &pinch::ZwpPointerGesturePinchV1,
        event: pinch::Event,
        name: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            pinch::Event::Begin {
                surface, fingers, ..
            } => state.begin(
                *name,
                surface.id(),
                fingers,
                TouchpadGestureDelta::Transform {
                    scale: 1.0,
                    rotation: 0.0,
                },
            ),
            pinch::Event::Update {
                scale, rotation, ..
            } => state.update(
                *name,
                TouchpadGestureDelta::Transform {
                    scale: scale as f32,
                    rotation: (rotation as f32).to_radians(),
                },
            ),
            pinch::Event::End { cancelled, .. } => state.finish(*name, cancelled != 0),
            _ => {}
        }
    }
}

wayland_client::delegate_noop!(GestureState: ignore gestures::ZwpPointerGesturesV1);
