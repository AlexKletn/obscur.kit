use obscur_geometry::Point;
use wayland_client::{Proxy, protocol::wl_pointer};
use wayland_protocols::wp::pointer_gestures::zv1::client::{
    zwp_pointer_gesture_pinch_v1 as pinch, zwp_pointer_gesture_swipe_v1 as swipe,
};

pub(super) struct GesturePointer {
    pub(super) proxy: wl_pointer::WlPointer,
    pub(super) swipe: swipe::ZwpPointerGestureSwipeV1,
    pub(super) pinch: pinch::ZwpPointerGesturePinchV1,
    pub(super) position: Point,
}

impl Drop for GesturePointer {
    fn drop(&mut self) {
        self.swipe.destroy();
        self.pinch.destroy();
        if self.proxy.version() >= 3 {
            self.proxy.release();
        }
    }
}
