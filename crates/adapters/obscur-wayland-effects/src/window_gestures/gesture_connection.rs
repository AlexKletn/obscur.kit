use std::io::ErrorKind;
use std::os::unix::net::UnixStream;

use obscur_interaction::gesture::TouchpadGestureEvent;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use rustix::event::{PollFd, PollFlags, poll};
use wayland_client::globals::{GlobalList, registry_queue_init};
use wayland_client::protocol::wl_seat;
use wayland_client::{EventQueue, Proxy};
use wayland_protocols::wp::pointer_gestures::zv1::client::zwp_pointer_gestures_v1 as gestures;

use crate::window_blur::window_surface;

use super::{PointingCallback, gesture_state::GestureState};

pub(super) struct GestureConnection {
    globals: Option<GlobalList>,
    state: GestureState,
    queue: EventQueue<GestureState>,
}

impl GestureConnection {
    pub(super) unsafe fn new(
        window: &(impl HasDisplayHandle + HasWindowHandle),
        monitor_pointing: bool,
    ) -> Result<Option<Self>, String> {
        // SAFETY: the caller retains the window for this guest connection's lifetime.
        let Some((connection, surface)) = (unsafe { window_surface(window) })? else {
            return Ok(None);
        };
        let (globals, queue) =
            registry_queue_init::<GestureState>(&connection).map_err(|error| error.to_string())?;
        let handle = queue.handle();
        let manager = globals
            .bind::<gestures::ZwpPointerGesturesV1, _, _>(&handle, 1..=3, ())
            .ok();
        if manager.is_none() && !monitor_pointing {
            globals.destroy();
            return Ok(None);
        }
        let mut state = GestureState::new(surface.id(), manager);
        for global in globals.contents().clone_list() {
            if global.interface == wl_seat::WlSeat::interface().name {
                state.add_seat(globals.registry(), &handle, global.name, global.version);
            }
        }
        Ok(Some(Self {
            queue,
            globals: Some(globals),
            state,
        }))
    }

    pub(super) fn run(
        &mut self,
        stop: &UnixStream,
        deliver: &mut impl FnMut(TouchpadGestureEvent) -> bool,
        pointing: &mut Option<PointingCallback>,
    ) -> Result<(), String> {
        let mut last = None;
        loop {
            self.queue
                .dispatch_pending(&mut self.state)
                .map_err(|error| error.to_string())?;
            if let Some(report) = pointing.as_mut() {
                let capabilities = self.state.pointing_capabilities();
                if last != Some(capabilities) {
                    last = Some(capabilities);
                    if !report(capabilities) {
                        return Ok(());
                    }
                }
            }
            if !self.state.take_events().into_iter().all(&mut *deliver) {
                return Ok(());
            }
            self.queue.flush().map_err(|error| error.to_string())?;
            // The guest queue receives messages even when winit has no events
            // to dispatch. A coordinated read wakes the host for gesture input.
            let Some(read) = self.queue.prepare_read() else {
                continue;
            };
            let mut fds = [
                PollFd::from_borrowed_fd(read.connection_fd(), PollFlags::IN),
                PollFd::new(stop, PollFlags::IN),
            ];
            match poll(&mut fds, None) {
                Ok(_) if !fds[1].revents().is_empty() => return Ok(()),
                Ok(_) => {}
                Err(rustix::io::Errno::INTR) => continue,
                Err(error) => return Err(error.to_string()),
            }
            match read.read() {
                Ok(_) => {}
                Err(wayland_client::backend::WaylandError::Io(error))
                    if error.kind() == ErrorKind::WouldBlock => {}
                Err(error) => return Err(error.to_string()),
            }
        }
    }
}

impl Drop for GestureConnection {
    fn drop(&mut self) {
        if let Some(globals) = self.globals.take() {
            globals.destroy();
        }
    }
}
