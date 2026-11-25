//! An activation from a native event on a mapped panel.

use wayland_client::protocol::{wl_seat::WlSeat, wl_surface::WlSurface};

/// Single-use Wayland activation for the popup opened in response to a panel input event.
#[derive(Debug)]
pub struct PopupActivation {
    pub(crate) seat: WlSeat,
    pub(crate) serial: u32,
    pub(crate) parent: WlSurface,
}
