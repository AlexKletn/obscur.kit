use wayland_client::{Proxy, protocol::wl_seat};

use super::gesture_pointer::GesturePointer;

pub(super) struct GestureSeat {
    pub(super) proxy: wl_seat::WlSeat,
    pub(super) pointer: Option<GesturePointer>,
    pub(super) capabilities: Option<crate::WaylandPointingCapabilities>,
}

impl Drop for GestureSeat {
    fn drop(&mut self) {
        self.pointer = None;
        if self.proxy.version() >= 5 {
            self.proxy.release();
        }
    }
}
