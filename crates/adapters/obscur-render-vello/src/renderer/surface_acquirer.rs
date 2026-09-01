#![doc = include_str!("surface_acquirer.doc/_module.md")]

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread::JoinHandle;

use crate::error::VelloRenderError;

use super::acquired_image::AcquiredImage;
use super::vello_surface::VelloSurface;

#[doc = include_str!("surface_acquirer.doc/AcquireWake.md")]
pub type AcquireWake = Arc<dyn Fn() + Send + Sync + 'static>;

#[doc = include_str!("surface_acquirer.doc/ReadySlot.md")]
type ReadySlot = Arc<Mutex<Option<AcquiredImage>>>;

#[doc = include_str!("surface_acquirer.doc/AcquireRequest.md")]
enum AcquireRequest {
    Image,
    Resize { width: u32, height: u32 },
    Stop,
}

#[doc = include_str!("surface_acquirer.doc/SurfaceAcquirer.md")]
pub struct SurfaceAcquirer {
    requests: SyncSender<AcquireRequest>,
    #[doc = include_str!("surface_acquirer.doc/ready.md")]
    ready: ReadySlot,
    #[doc = include_str!("surface_acquirer.doc/outstanding.md")]
    outstanding: bool,
    thread: Option<JoinHandle<()>>,
}

impl SurfaceAcquirer {
    #[doc = include_str!("surface_acquirer.doc/spawn.md")]
    #[must_use]
    pub fn spawn(surface: VelloSurface, device: wgpu::Device, wake: Option<AcquireWake>) -> Self {
        // One outstanding image, and one pending request. A deeper queue would
        // only let the loop run further ahead of a display it cannot outrun.
        let (requests, inbox) = sync_channel::<AcquireRequest>(1);
        let ready: ReadySlot = Arc::new(Mutex::new(None));
        let held = Arc::clone(&ready);

        let thread = std::thread::Builder::new()
            .name("obscur-surface-acquire".to_owned())
            .spawn(move || acquire_loop(surface, &device, &inbox, &held, wake.as_ref()))
            .ok();

        Self {
            requests,
            ready,
            outstanding: false,
            thread,
        }
    }

    #[doc = include_str!("surface_acquirer.doc/request.md")]
    pub fn request(&mut self) {
        if self.outstanding {
            return;
        }
        // `try_send`, never `send`: this is called from the loop that reads
        // input, and the whole point of this type is that that loop does not
        // block. A full channel means a request is already in flight, which is
        // the same state `outstanding` records.
        if self.requests.try_send(AcquireRequest::Image).is_ok() {
            self.outstanding = true;
        }
    }

    #[doc = include_str!("surface_acquirer.doc/try_take.md")]
    pub fn try_take(&mut self) -> Option<AcquiredImage> {
        let taken = self
            .ready
            .lock()
            .unwrap_or_else(|held| held.into_inner())
            .take();
        if taken.is_some() {
            self.outstanding = false;
        }
        taken
    }

    #[doc = include_str!("surface_acquirer.doc/resize.md")]
    pub fn resize(&mut self, width: u32, height: u32) {
        // Dropping a ready image on the floor: it belongs to the old size and
        // presenting it after a resize is a stretched frame. The acquire thread
        // clears the same slot again before it configures, which is what closes
        // the race this side cannot: an acquire still in flight lands AFTER
        // this call, and `Surface::configure` with a live `SurfaceOutput` is a
        // wgpu validation error — raised as a panic, on a fractionally scaled
        // desktop surface, about half the times the process started.
        self.clear();
        self.outstanding = false;
        let _ = self
            .requests
            .try_send(AcquireRequest::Resize { width, height });
    }

    #[doc = include_str!("surface_acquirer.doc/clear.md")]
    fn clear(&self) {
        drop(
            self.ready
                .lock()
                .unwrap_or_else(|held| held.into_inner())
                .take(),
        );
    }

    #[doc = include_str!("surface_acquirer.doc/is_waiting.md")]
    #[must_use]
    pub const fn is_waiting(&self) -> bool {
        self.outstanding
    }
}

impl Drop for SurfaceAcquirer {
    fn drop(&mut self) {
        // Free whatever is held so nothing outlives the surface it came from,
        // then ask the thread to stop. It may be inside a blocking acquire; it
        // will see the stop when that returns, which is within a frame on any
        // live surface.
        self.clear();
        let _ = self.requests.try_send(AcquireRequest::Stop);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl std::fmt::Debug for SurfaceAcquirer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurfaceAcquirer")
            .field("outstanding", &self.outstanding)
            .field("live", &self.thread.is_some())
            .finish_non_exhaustive()
    }
}

#[doc = include_str!("surface_acquirer.doc/acquire_loop.md")]
fn acquire_loop(
    mut surface: VelloSurface,
    device: &wgpu::Device,
    inbox: &Receiver<AcquireRequest>,
    ready: &ReadySlot,
    wake: Option<&AcquireWake>,
) {
    let take = || ready.lock().unwrap_or_else(|held| held.into_inner()).take();
    while let Ok(request) = inbox.recv() {
        match request {
            AcquireRequest::Stop => {
                // Nothing this thread handed out may outlive the surface it was
                // taken from.
                drop(take());
                return;
            }
            AcquireRequest::Resize { width, height } => {
                // An image acquired since the caller cleared the slot is still
                // alive, and reconfiguring under it is the panic this reclaim
                // exists to prevent. It is discarded rather than kept: it
                // belongs to the extent being replaced.
                drop(take());
                surface.resize(device, width, height);
            }
            AcquireRequest::Image => {
                // THE blocking call, and the only reason this thread exists. On
                // a `Fifo` surface it waits for the compositor to release a
                // buffer — hundreds of milliseconds under load — and every one
                // of those milliseconds used to be spent on the thread that
                // reads input.
                match surface.current_texture() {
                    Ok(texture) => {
                        // The extent travels with the image. The caller cannot
                        // ask the surface for it — the surface is here — and a
                        // blit against the wrong extent is a stretched frame.
                        let image = AcquiredImage {
                            texture,
                            width: surface.config().width,
                            height: surface.config().height,
                            format: surface.blit_format(),
                        };
                        *ready.lock().unwrap_or_else(|held| held.into_inner()) = Some(image);
                        if let Some(wake) = wake {
                            // The loop is asleep in its own wait; without this it
                            // would not know the image had landed, and asking it
                            // to poll instead would be the busy loop §3.5 forbids.
                            wake();
                        }
                    }
                    Err(VelloRenderError::SurfaceLost(_)) => {
                        // Reconfigure and let the next request try again. There
                        // is nobody here to report to: the renderer owns the
                        // diagnostics and it is on the other thread.
                        surface.configure(device);
                    }
                    Err(_) => return,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A live `VelloSurface` needs a window and a GPU, so what is checked here is
    // the part that has neither and still breaks: the request bookkeeping that
    // decides whether the loop blocks, and the slot that must be empty before
    // anything reconfigures the surface.

    #[test]
    fn a_second_request_while_one_is_outstanding_is_not_sent_twice() {
        let (requests, inbox) = sync_channel::<AcquireRequest>(1);
        let ready: ReadySlot = Arc::new(Mutex::new(None));
        let mut acquirer = SurfaceAcquirer {
            requests,
            ready,
            outstanding: false,
            thread: None,
        };

        acquirer.request();
        assert!(acquirer.is_waiting());
        acquirer.request();
        acquirer.request();

        let mut queued = 0;
        while inbox.try_recv().is_ok() {
            queued += 1;
        }
        assert_eq!(queued, 1, "one image asked for, however often it is asked");
    }

    #[test]
    fn a_request_that_cannot_be_sent_does_not_claim_to_be_outstanding() {
        // The channel is full because the thread has not taken the last request.
        // Recording that as outstanding would leave the loop believing an image
        // is coming that nobody is fetching.
        let (requests, _inbox) = sync_channel::<AcquireRequest>(1);
        let ready: ReadySlot = Arc::new(Mutex::new(None));
        let mut acquirer = SurfaceAcquirer {
            requests,
            ready,
            outstanding: false,
            thread: None,
        };
        acquirer.request();
        assert!(acquirer.is_waiting());

        // Fill it a second time by hand, then clear the flag as a fresh loop
        // turn would see it, and ask again.
        acquirer.outstanding = false;
        acquirer.request();
        assert!(
            !acquirer.is_waiting(),
            "the send failed, so nothing is in flight"
        );
    }

    #[test]
    fn a_resize_empties_the_slot_the_thread_reconfigures_under() {
        // The other half is on the acquire thread — an image that lands after
        // this call is reclaimed there — but an image already sitting in the
        // slot has to be gone before the resize is even asked for.
        let (requests, _inbox) = sync_channel::<AcquireRequest>(1);
        let ready: ReadySlot = Arc::new(Mutex::new(None));
        let acquirer = SurfaceAcquirer {
            requests,
            ready: Arc::clone(&ready),
            outstanding: true,
            thread: None,
        };
        acquirer.clear();
        assert!(
            ready.lock().unwrap().is_none(),
            "nothing acquired may survive a reconfigure"
        );
    }

    #[test]
    fn taking_nothing_leaves_the_request_in_flight() {
        let (requests, _inbox) = sync_channel::<AcquireRequest>(1);
        let ready: ReadySlot = Arc::new(Mutex::new(None));
        let mut acquirer = SurfaceAcquirer {
            requests,
            ready,
            outstanding: false,
            thread: None,
        };
        acquirer.request();
        assert!(acquirer.try_take().is_none());
        assert!(
            acquirer.is_waiting(),
            "no image yet is not the same as no image coming"
        );
    }

    #[test]
    fn a_resize_cancels_what_was_in_flight() {
        // An image acquired at the old size would present stretched, so the
        // resize path must not leave the loop thinking one is on its way.
        let (requests, inbox) = sync_channel::<AcquireRequest>(1);
        let ready: ReadySlot = Arc::new(Mutex::new(None));
        let mut acquirer = SurfaceAcquirer {
            requests,
            ready,
            outstanding: true,
            thread: None,
        };
        acquirer.resize(800, 600);
        assert!(!acquirer.is_waiting());
        assert!(matches!(
            inbox.try_recv(),
            Ok(AcquireRequest::Resize {
                width: 800,
                height: 600
            })
        ));
    }
}
