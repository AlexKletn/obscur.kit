//! Compositor-owned movement using the serial of a live Wayland touch contact.

mod touch_move_seat;
mod touch_move_state;

use std::ffi::c_void;
use std::ptr::NonNull;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use wayland_client::backend::ObjectId;
use wayland_client::globals::{GlobalList, registry_queue_init};
use wayland_client::protocol::wl_seat;
use wayland_client::{EventQueue, Proxy};
use wayland_protocols::xdg::shell::client::xdg_toplevel::XdgToplevel;

use crate::window_blur::window_surface;
use touch_move_state::TouchMoveState;

/// Owns the guest queue that receives the compositor's touch serials.
pub struct TouchWindowMove<W> {
    globals: Option<GlobalList>,
    queue: EventQueue<TouchMoveState>,
    state: TouchMoveState,
    toplevel: XdgToplevel,
    _window: W,
}

impl<W: HasDisplayHandle + HasWindowHandle> TouchWindowMove<W> {
    /// Borrows an existing window and its xdg-toplevel on the window thread.
    ///
    /// # Safety
    ///
    /// The toplevel must be the live Wayland proxy belonging to `window`.
    pub unsafe fn new(window: W, toplevel: NonNull<c_void>) -> Result<Option<Self>, String> {
        // SAFETY: the owned window outlives the guest queue and its proxies.
        let Some((connection, surface)) = (unsafe { window_surface(&window) })? else {
            return Ok(None);
        };
        // SAFETY: the caller supplies the toplevel belonging to this live window.
        let id = unsafe { ObjectId::from_ptr(XdgToplevel::interface(), toplevel.as_ptr().cast()) }
            .map_err(|error| error.to_string())?;
        let toplevel = XdgToplevel::from_id(&connection, id).map_err(|error| error.to_string())?;
        let (globals, queue) = registry_queue_init::<TouchMoveState>(&connection)
            .map_err(|error| error.to_string())?;
        let mut state = TouchMoveState::new(surface.id());
        for global in globals.contents().clone_list() {
            if global.interface == wl_seat::WlSeat::interface().name {
                state.add_seat(
                    globals.registry(),
                    &queue.handle(),
                    global.name,
                    global.version,
                );
            }
        }
        let mut mover = Self {
            globals: Some(globals),
            queue,
            state,
            toplevel,
            _window: window,
        };
        mover
            .queue
            .roundtrip(&mut mover.state)
            .map_err(|error| error.to_string())?;
        mover.refresh()?;
        Ok(Some(mover))
    }

    /// Requests a move for the initial contact delivered to this surface.
    pub fn begin(&mut self, contact: u64) -> Result<bool, String> {
        self.refresh()?;
        let Some((seat, serial)) = self.state.take_press(contact as i32) else {
            return Ok(false);
        };
        self.toplevel._move(&seat, serial);
        self.queue.flush().map_err(|error| error.to_string())?;
        Ok(true)
    }

    /// Drains input and capability changes enqueued by the window's display read.
    pub fn refresh(&mut self) -> Result<(), String> {
        self.queue
            .dispatch_pending(&mut self.state)
            .map_err(|error| error.to_string())?;
        self.queue.flush().map_err(|error| error.to_string())
    }
}

impl<W> Drop for TouchWindowMove<W> {
    fn drop(&mut self) {
        if let Some(globals) = self.globals.take() {
            globals.destroy();
        }
    }
}
