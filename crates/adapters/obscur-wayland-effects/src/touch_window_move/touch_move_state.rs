use std::collections::HashMap;
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle, WEnum,
    backend::ObjectId,
    globals::GlobalListContents,
    protocol::{wl_registry, wl_seat, wl_touch},
};

use super::touch_move_seat::TouchMoveSeat;

pub(super) struct TouchMoveState {
    surface: ObjectId,
    seats: HashMap<u32, TouchMoveSeat>,
}

impl TouchMoveState {
    pub(super) fn new(surface: ObjectId) -> Self {
        Self {
            surface,
            seats: HashMap::new(),
        }
    }

    pub(super) fn add_seat(
        &mut self,
        registry: &wl_registry::WlRegistry,
        queue: &QueueHandle<Self>,
        name: u32,
        version: u32,
    ) {
        self.seats.entry(name).or_insert_with(|| TouchMoveSeat {
            seat: registry.bind(name, version.min(7), queue, name),
            touch: None,
            presses: HashMap::new(),
        });
    }

    pub(super) fn take_press(&mut self, contact: i32) -> Option<(wl_seat::WlSeat, u32)> {
        let mut matching = self.seats.values_mut().filter_map(|seat| {
            seat.presses
                .remove(&contact)
                .map(|serial| (seat.seat.clone(), serial))
        });
        let press = matching.next()?;
        matching.next().is_none().then_some(press)
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for TouchMoveState {
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
                state.add_seat(registry, queue, name, version)
            }
            wl_registry::Event::GlobalRemove { name } => {
                state.seats.remove(&name);
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_seat::WlSeat, u32> for TouchMoveState {
    fn event(
        state: &mut Self,
        _: &wl_seat::WlSeat,
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
        if capabilities.contains(wl_seat::Capability::Touch) {
            seat.touch
                .get_or_insert_with(|| seat.seat.get_touch(queue, *name));
        } else {
            seat.release_touch();
        }
    }
}

impl Dispatch<wl_touch::WlTouch, u32> for TouchMoveState {
    fn event(
        state: &mut Self,
        touch: &wl_touch::WlTouch,
        event: wl_touch::Event,
        name: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let Some(seat) = state
            .seats
            .get_mut(name)
            .filter(|seat| seat.touch.as_ref() == Some(touch))
        else {
            return;
        };
        match event {
            wl_touch::Event::Down {
                surface,
                id,
                serial,
                ..
            } if surface.id() == state.surface => {
                seat.presses.insert(id, serial);
            }
            wl_touch::Event::Up { id, .. } => {
                seat.presses.remove(&id);
            }
            wl_touch::Event::Cancel => seat.presses.clear(),
            _ => {}
        }
    }
}
