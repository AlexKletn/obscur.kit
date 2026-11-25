//! Acquire and present swapchain images off the window event thread.

mod acquire_request;
mod acquire_state;

use acquire_request::AcquireRequest;
use acquire_state::{AcquireState, SharedAcquireState, lock};

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::error::VelloRenderError;

use super::acquired_image::AcquiredImage;
use super::vello_surface::VelloSurface;

#[doc = include_str!("../../docs/api/renderer/surface_acquirer.doc/AcquireWake.md")]
pub type AcquireWake = Arc<dyn Fn() + Send + Sync + 'static>;

#[doc = include_str!("../../docs/api/renderer/surface_acquirer.doc/SurfaceAcquirer.md")]
pub struct SurfaceAcquirer {
    /// Nonblocking request sender. Dropping it disconnects the worker, including when an image request already fills the queue.
    requests: Option<SyncSender<AcquireRequest>>,
    /// The state this side shares with the acquire thread.
    shared: SharedAcquireState,
    thread: Option<JoinHandle<()>>,
    startup_error: Option<std::io::Error>,
}

impl SurfaceAcquirer {
    #[doc = include_str!("../../docs/api/renderer/surface_acquirer.doc/spawn.md")]
    #[must_use]
    pub fn spawn(surface: VelloSurface, device: wgpu::Device, wake: Option<AcquireWake>) -> Self {
        // One outstanding image, and one pending request. A deeper queue would
        // only let the loop run further ahead of a display it cannot outrun.
        let (requests, inbox) = sync_channel::<AcquireRequest>(1);
        // Read before the surface moves onto the thread. Seeding this is what
        // makes the first redundant resize recognisable as redundant; starting
        // at `None` would let one storm through before the guard could bite.
        let started_at = (surface.config().width, surface.config().height);
        let shared: SharedAcquireState = Arc::new(Mutex::new(AcquireState {
            extent: Some(started_at),
            ..AcquireState::default()
        }));
        let held = Arc::clone(&shared);

        let thread = std::thread::Builder::new()
            .name("obscur-surface-acquire".to_owned())
            .spawn(move || acquire_loop(surface, &device, &inbox, &held, wake.as_ref()));

        Self::from_worker(requests, shared, thread)
    }

    fn from_worker(
        requests: SyncSender<AcquireRequest>,
        shared: SharedAcquireState,
        worker: std::io::Result<JoinHandle<()>>,
    ) -> Self {
        let (requests, thread, startup_error) = match worker {
            Ok(thread) => (Some(requests), Some(thread), None),
            Err(error) => (None, None, Some(error)),
        };
        Self {
            requests,
            shared,
            thread,
            startup_error,
        }
    }

    /// Takes the OS error if the acquisition thread failed to start.
    pub fn take_startup_error(&mut self) -> Option<std::io::Error> {
        self.startup_error.take()
    }

    #[doc = include_str!("../../docs/api/renderer/surface_acquirer.doc/request.md")]
    pub fn request(&mut self) {
        let Some(requests) = self.requests.as_ref() else {
            return;
        };
        {
            let mut state = lock(&self.shared);
            // Both halves are the same question — is this side already owed an
            // image — and both have to be asked under the lock the acquire thread
            // answers them with. An image already in the slot counts: asking for
            // a second one while the first is still out is
            // `get_current_texture` on a surface whose image has not been
            // presented, which wgpu calls `Surface image is already acquired`.
            if state.in_flight || state.ready.is_some() || state.presented_at.is_some() {
                return;
            }
            state.in_flight = true;
        }
        // `try_send`, never `send`: this is called from the loop that reads
        // input, and the whole point of this type is that that loop does not
        // block. A send that fails leaves nothing in flight, so the claim above
        // is taken back rather than left standing.
        if requests.try_send(AcquireRequest::Image).is_err() {
            lock(&self.shared).in_flight = false;
        }
    }

    #[doc = include_str!("../../docs/api/renderer/surface_acquirer.doc/try_take.md")]
    pub fn try_take(&mut self) -> Option<AcquiredImage> {
        self.try_take_timed().map(|(image, _)| image)
    }

    /// Take the ready image and the time the worker spent preparing the swapchain and
    /// acquiring it.
    pub fn try_take_timed(&mut self) -> Option<(AcquiredImage, Duration)> {
        lock(&self.shared).ready.take()
    }

    /// Stage the latest extent for the next image request and discard any ready image
    /// at the superseded extent. Several extent changes can share one configure and
    /// acquire turn. An acquire already running remains outstanding until the worker
    /// finishes it.
    pub fn resize(&mut self, width: u32, height: u32) {
        let mut state = lock(&self.shared);
        let extent = (width, height);
        if state.resize_to.or(state.extent) == Some(extent) {
            return;
        }
        drop(state.ready.take());
        state.resize_to = Some(extent);
    }

    /// Hand an acquired surface texture back to the worker for presentation. No new
    /// acquire or configure starts until the presentation has completed and its
    /// timestamp has been taken. On failure, the caller gets the texture back.
    pub fn queue_present(
        &mut self,
        texture: wgpu::SurfaceTexture,
    ) -> Result<(), wgpu::SurfaceTexture> {
        let Some(requests) = self.requests.as_ref() else {
            return Err(texture);
        };
        {
            let mut state = lock(&self.shared);
            if state.in_flight || state.ready.is_some() || state.presented_at.is_some() {
                return Err(texture);
            }
            state.in_flight = true;
        }

        match requests.try_send(AcquireRequest::Present(texture)) {
            Ok(()) => Ok(()),
            Err(
                std::sync::mpsc::TrySendError::Full(AcquireRequest::Present(texture))
                | std::sync::mpsc::TrySendError::Disconnected(AcquireRequest::Present(texture)),
            ) => {
                lock(&self.shared).in_flight = false;
                Err(texture)
            }
            Err(_) => unreachable!("only a present request was sent"),
        }
    }

    /// Take the completion timestamp of a queued presentation.
    pub fn take_presented_at(&mut self) -> Option<Instant> {
        lock(&self.shared).presented_at.take()
    }

    /// Whether an acquire is running right now, as the acquire thread last left it.
    #[must_use]
    pub fn is_waiting(&self) -> bool {
        lock(&self.shared).in_flight
    }
}

impl Drop for SurfaceAcquirer {
    fn drop(&mut self) {
        // Free whatever is held so nothing outlives the surface it came from,
        // and say so under the lock, so an acquire that lands in the meantime is
        // dropped on the acquire thread rather than parked in a slot nobody will
        // empty.
        {
            let mut state = lock(&self.shared);
            state.stopping = true;
            drop(state.ready.take());
            // Disconnect before joining, even if an image request fills the queue.
            self.requests = None;
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl std::fmt::Debug for SurfaceAcquirer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurfaceAcquirer")
            .field("in_flight", &self.is_waiting())
            .field("live", &self.thread.is_some())
            .finish_non_exhaustive()
    }
}

/// Configure the swapchain to the latest requested extent immediately before
/// acquiring an image.
fn apply_pending_resize(
    surface: &mut VelloSurface,
    device: &wgpu::Device,
    shared: &SharedAcquireState,
) {
    let pending = {
        let mut state = lock(shared);
        let pending = state.resize_to.take();
        if pending.is_some() {
            // Invalidate the cached extent while configure runs so a resize back
            // to the previous size can still supersede the in-flight size.
            state.extent = None;
        }
        pending
    };
    if let Some((width, height)) = pending {
        if (surface.config().width, surface.config().height) != (width, height) {
            surface.resize(device, width, height);
        }
        // After the reconfigure and not before: until `VelloSurface::resize`
        // returns, the swapchain is still at the old extent, and a guard reading
        // the new one would wave through an image taken from the old.
        lock(shared).extent = Some((width, height));
    }
}

#[doc = include_str!("../../docs/api/renderer/surface_acquirer.doc/acquire_loop.md")]
fn acquire_loop(
    mut surface: VelloSurface,
    device: &wgpu::Device,
    inbox: &Receiver<AcquireRequest>,
    shared: &SharedAcquireState,
    wake: Option<&AcquireWake>,
) {
    while let Ok(request) = inbox.recv() {
        if lock(shared).stopping {
            break;
        }
        if let AcquireRequest::Present(texture) = request {
            texture.present();
            {
                let mut state = lock(shared);
                state.presented_at = Some(Instant::now());
                state.in_flight = false;
            }
            if let Some(wake) = wake {
                wake();
            }
            continue;
        }
        // Configure and acquire in one worker turn. Resize events that arrive
        // before this turn replace the requested extent without reconfiguring.
        let started = Instant::now();
        apply_pending_resize(&mut surface, device, shared);
        match surface.current_texture() {
            Ok(texture) => {
                let elapsed = started.elapsed();
                // The extent travels with the image. The caller cannot ask the
                // surface for it — the surface is here — and a blit against the
                // wrong extent is a stretched frame.
                let image = AcquiredImage {
                    texture,
                    width: surface.config().width,
                    height: surface.config().height,
                    format: surface.blit_format(),
                };
                {
                    let mut state = lock(shared);
                    state.in_flight = false;
                    if state.stopping || state.resize_to.is_some() {
                        // Taken from an extent that is already history, or from a
                        // surface that is going away: it belongs to nobody.
                        // Handing it over would cost a turn to reject it there;
                        // dropping it here returns it to the swapchain at once.
                        drop(image);
                    } else {
                        state.ready = Some((image, elapsed));
                    }
                }
                if let Some(wake) = wake {
                    // The loop is asleep in its own wait; without this it would
                    // not know the image had landed, and asking it to poll
                    // instead would be the busy loop §3.5 forbids. Rung even when
                    // the image was dropped: the loop still has to come back and
                    // ask again, or the surface stays dark until something else
                    // wakes it.
                    wake();
                }
            }
            Err(VelloRenderError::SurfaceLost(_)) => {
                // Reconfigure and let the next request try again. There is
                // nobody here to report to: the renderer owns the diagnostics
                // and it is on the other thread.
                lock(shared).in_flight = false;
                surface.configure(device);
                if let Some(wake) = wake {
                    wake();
                }
            }
            Err(_) => {
                lock(shared).in_flight = false;
                break;
            }
        }
    }
    // Nothing this thread handed out may outlive the surface it was taken from,
    // and the surface is dropped the moment this function returns.
    drop(lock(shared).ready.take());
}

#[cfg(test)]
mod tests {
    use super::*;

    // A live `VelloSurface` needs a window and a GPU, so what is checked here is
    // the part that has neither and still breaks: the bookkeeping that decides
    // whether a second acquire can be asked for while the first is out, and the
    // resize that has to survive a channel which would not take it.

    fn detached() -> (SurfaceAcquirer, Receiver<AcquireRequest>) {
        let (requests, inbox) = sync_channel::<AcquireRequest>(1);
        let acquirer = SurfaceAcquirer {
            requests: Some(requests),
            shared: Arc::new(Mutex::new(AcquireState::default())),
            thread: None,
            startup_error: None,
        };
        (acquirer, inbox)
    }

    #[test]
    fn a_failed_worker_start_retains_the_error_and_closes_requests() {
        let (requests, inbox) = sync_channel(1);
        let mut acquirer = SurfaceAcquirer::from_worker(
            requests,
            Arc::new(Mutex::new(AcquireState::default())),
            Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied)),
        );

        assert_eq!(
            acquirer.take_startup_error().unwrap().kind(),
            std::io::ErrorKind::PermissionDenied
        );
        assert_eq!(
            inbox.try_recv().err(),
            Some(std::sync::mpsc::TryRecvError::Disconnected)
        );
        acquirer.request();
        assert!(!acquirer.is_waiting());
    }

    #[test]
    fn dropping_an_acquirer_disconnects_a_full_queue_before_joining() {
        let (mut acquirer, inbox) = detached();
        acquirer.request();
        let (release_worker, wait_for_drop) = sync_channel(0);
        let shared = Arc::clone(&acquirer.shared);
        let worker = std::thread::spawn(move || {
            wait_for_drop.recv().expect("test worker released");
            let deadline = Instant::now() + Duration::from_secs(2);
            while !lock(&shared).stopping {
                assert!(Instant::now() < deadline, "shutdown did not start");
                std::thread::yield_now();
            }
            assert!(matches!(inbox.recv(), Ok(AcquireRequest::Image)));
            assert!(inbox.recv().is_err(), "the sender was disconnected");
        });
        acquirer.thread = Some(worker);

        let (done, completion) = sync_channel(1);
        let dropping = std::thread::spawn(move || {
            drop(acquirer);
            let _ = done.send(());
        });
        release_worker.send(()).expect("worker still waiting");
        completion
            .recv_timeout(Duration::from_secs(2))
            .expect("dropping the acquirer joins the worker");
        dropping.join().expect("drop thread completed");
    }

    #[test]
    fn a_present_completion_is_observed_before_the_next_acquire() {
        let (mut acquirer, inbox) = detached();
        let presented_at = Instant::now();
        lock(&acquirer.shared).presented_at = Some(presented_at);

        acquirer.resize(1024, 768);
        acquirer.request();
        assert!(inbox.try_recv().is_err());

        assert_eq!(acquirer.take_presented_at(), Some(presented_at));
        acquirer.request();
        assert!(matches!(inbox.try_recv(), Ok(AcquireRequest::Image)));
        assert_eq!(lock(&acquirer.shared).resize_to, Some((1024, 768)));
    }

    #[test]
    fn a_second_request_while_one_is_outstanding_is_not_sent_twice() {
        let (mut acquirer, inbox) = detached();

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
        let (mut acquirer, _inbox) = detached();
        acquirer.request();
        assert!(acquirer.is_waiting());

        lock(&acquirer.shared).in_flight = false;
        acquirer.request();
        assert!(
            !acquirer.is_waiting(),
            "the send failed, so nothing is in flight"
        );
    }

    #[test]
    fn a_resize_empties_the_slot_the_thread_reconfigures_under() {
        let (mut acquirer, _inbox) = detached();
        acquirer.resize(800, 600);
        let state = lock(&acquirer.shared);
        assert!(state.ready.is_none());
        assert_eq!(state.resize_to, Some((800, 600)));
    }

    #[test]
    fn taking_nothing_leaves_the_request_in_flight() {
        let (mut acquirer, _inbox) = detached();
        acquirer.request();
        assert!(acquirer.try_take().is_none());
        assert!(
            acquirer.is_waiting(),
            "no image yet is not the same as no image coming"
        );
    }

    #[test]
    fn a_resize_does_not_cancel_an_acquire_it_cannot_see() {
        // THE regression this file was rewritten for. `resize` used to clear the
        // in-flight flag outright, and the acquire it was clearing had not
        // stopped running — it was inside `get_current_texture`. The next turn
        // then asked for a second image while the first was still being fetched,
        // and wgpu answers that with `Surface image is already acquired`,
        // followed by a panic tearing the swapchain down under a live texture.
        // Nothing on this side may declare an acquire it did not start finished.
        let (mut acquirer, _inbox) = detached();
        acquirer.request();
        assert!(acquirer.is_waiting());

        acquirer.resize(800, 600);
        assert!(
            acquirer.is_waiting(),
            "the acquire is still running; only the thread may say it is not"
        );
    }

    #[test]
    fn successive_resizes_share_one_acquire_and_keep_the_latest_extent() {
        let (mut acquirer, inbox) = detached();
        acquirer.resize(800, 600);
        acquirer.resize(900, 650);
        assert!(inbox.try_recv().is_err());

        acquirer.request();
        acquirer.resize(1024, 768);
        assert!(matches!(inbox.try_recv(), Ok(AcquireRequest::Image)));
        assert!(inbox.try_recv().is_err());
        assert_eq!(lock(&acquirer.shared).resize_to, Some((1024, 768)));
    }
}
