//! Presentation feedback on a window owned by another Wayland client integration.

use std::time::{Duration, Instant};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use wayland_client::globals::registry_queue_init;
use wayland_client::{EventQueue, protocol::wl_surface::WlSurface};
use wayland_protocols::wp::presentation_time::client::wp_presentation::WpPresentation;

use crate::presentation_feedback::PresentationFeedback;
use crate::window_blur::window_surface;

/// Pairs application frame identifiers with compositor presentation timestamps.
pub struct WindowPresentation<W> {
    presentation: WpPresentation,
    surface: WlSurface,
    queue: EventQueue<PresentationFeedback>,
    feedback: PresentationFeedback,
    _window: W,
}

impl<W: HasDisplayHandle + HasWindowHandle> WindowPresentation<W> {
    /// Bind presentation feedback when the window and compositor support it.
    pub fn new(window: W) -> Result<Option<Self>, String> {
        // SAFETY: the stored window outlives the presentation proxies and queue.
        let Some((connection, surface)) = (unsafe { window_surface(&window) })? else {
            return Ok(None);
        };
        let (globals, queue) = registry_queue_init::<PresentationFeedback>(&connection)
            .map_err(|error| error.to_string())?;
        let presentation = globals.bind(&queue.handle(), 1..=2, ());
        globals.destroy();
        let Ok(presentation) = presentation else {
            return Ok(None);
        };
        Ok(Some(Self {
            presentation,
            surface,
            queue,
            feedback: PresentationFeedback::default(),
            _window: window,
        }))
    }

    /// Attach feedback to the next commit on the window surface.
    pub fn request(&self, frame: u64) {
        self.presentation
            .feedback(&self.surface, &self.queue.handle(), frame);
    }

    /// Samples the Linux monotonic clock and its corresponding Rust instant when the compositor advertises that clock for presentation timestamps.
    #[must_use]
    pub fn monotonic_sample(&self) -> Option<(Duration, Instant)> {
        if self.feedback.clock_id != Some(rustix::time::ClockId::Monotonic as u32) {
            return None;
        }
        let instant = Instant::now();
        let clock = rustix::time::clock_gettime(rustix::time::ClockId::Monotonic);
        let duration = Duration::new(
            clock.tv_sec.try_into().ok()?,
            clock.tv_nsec.try_into().ok()?,
        );
        Some((duration, instant))
    }

    /// Dispatch received events and drain frame identifiers and timestamps. A discarded frame has no presentation timestamp.
    pub fn poll(&mut self) -> Result<impl Iterator<Item = (u64, Option<u64>)> + '_, String> {
        self.queue
            .dispatch_pending(&mut self.feedback)
            .map_err(|error| error.to_string())?;
        Ok(self.feedback.frames.drain(..))
    }
}

impl<W> Drop for WindowPresentation<W> {
    fn drop(&mut self) {
        self.presentation.destroy();
    }
}
