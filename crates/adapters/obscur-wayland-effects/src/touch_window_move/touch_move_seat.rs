use std::collections::HashMap;
use wayland_client::{
    Proxy,
    protocol::{wl_seat, wl_touch},
};

pub(super) struct TouchMoveSeat {
    pub(super) seat: wl_seat::WlSeat,
    pub(super) touch: Option<wl_touch::WlTouch>,
    pub(super) presses: HashMap<i32, u32>,
}

impl TouchMoveSeat {
    pub(super) fn release_touch(&mut self) {
        self.presses.clear();
        if let Some(touch) = self.touch.take()
            && touch.version() >= 3
        {
            touch.release();
        }
    }
}

impl Drop for TouchMoveSeat {
    fn drop(&mut self) {
        self.release_touch();
        if self.seat.version() >= 5 {
            self.seat.release();
        }
    }
}
