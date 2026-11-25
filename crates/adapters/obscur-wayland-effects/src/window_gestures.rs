use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::thread::{self, JoinHandle};

use obscur_interaction::gesture::TouchpadGestureEvent;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
mod gesture_connection;
mod gesture_pointer;
mod gesture_seat;
mod gesture_state;

use gesture_connection::GestureConnection;

type PointingCallback = Box<dyn FnMut(Option<crate::WaylandPointingCapabilities>) -> bool + Send>;

/// Native swipe and transform events for a Wayland window.
pub struct WindowGestures<W> {
    stop: UnixStream,
    worker: Option<JoinHandle<()>>,
    _window: W,
}

impl<W: HasDisplayHandle + HasWindowHandle> WindowGestures<W> {
    /// Subscribes to pointer gestures when the compositor advertises the protocol. The delivery callback runs on the Wayland gesture worker and returns false when the receiver has closed. Events use logical surface coordinates.
    pub fn new(
        window: W,
        deliver: impl FnMut(TouchpadGestureEvent) -> bool + Send + 'static,
    ) -> Result<Option<Self>, String> {
        Self::start(window, deliver, None)
    }

    /// Reports seat capabilities even on Wayland compositors without gesture support.
    pub fn with_pointing_capabilities(
        window: W,
        deliver: impl FnMut(TouchpadGestureEvent) -> bool + Send + 'static,
        report: impl FnMut(Option<crate::WaylandPointingCapabilities>) -> bool + Send + 'static,
    ) -> Result<Option<Self>, String> {
        Self::start(window, deliver, Some(Box::new(report)))
    }

    fn start(
        window: W,
        mut deliver: impl FnMut(TouchpadGestureEvent) -> bool + Send + 'static,
        mut pointing: Option<PointingCallback>,
    ) -> Result<Option<Self>, String> {
        // SAFETY: Drop joins the worker before the retained window can be released.
        let Some(mut connection) =
            (unsafe { GestureConnection::new(&window, pointing.is_some()) })?
        else {
            return Ok(None);
        };
        let (stop, wake_stop) = UnixStream::pair().map_err(|error| error.to_string())?;
        let worker = thread::Builder::new()
            .name("obscur-wayland-gestures".into())
            .spawn(move || {
                if let Err(error) = connection.run(&wake_stop, &mut deliver, &mut pointing) {
                    eprintln!("Wayland input dispatch failed: {error}");
                    if let Some(report) = pointing.as_mut() {
                        report(None);
                    }
                }
            })
            .map_err(|error| error.to_string())?;
        Ok(Some(Self {
            stop,
            worker: Some(worker),
            _window: window,
        }))
    }
}

impl<W> Drop for WindowGestures<W> {
    fn drop(&mut self) {
        let _ = self.stop.shutdown(Shutdown::Both);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
