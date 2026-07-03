//! The [`WinitWindowBackend`] winit adapter implementing the [`WindowBackend`]
//! port (spec §6.1-6.6 windowing; §2.7 "Реализует `WindowBackend`").

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use obscur_capability::SupportLevel;
use obscur_identity::WindowId;
use obscur_windowing::chrome::{ChromeMode, DragRegion, ResizeRegion, SystemMenuRequest};
use obscur_windowing::display::{Display, DisplayEvent, DisplayId};
use obscur_windowing::port::{WindowBackend, WindowError, WindowFeature};
use obscur_windowing::presentation::{PresentationMode, SleepInhibition};
use obscur_windowing::request::{
    ActivateWindow, ActivationResult, CloseWindow, CreateWindow, UpdateWindow,
    WindowOperationResult,
};
use obscur_windowing::window::{Window, WindowEvent, WindowState};

use super::winit_event_loop::{BackendState, WinitAppHandler};

/// A [`WindowBackend`] backed by winit 0.30 (spec §6.1-6.6, §2.7).
///
/// # Deferred, callback-driven creation
///
/// winit 0.30 is callback-driven: a real [`winit::window::Window`] can only be
/// built inside an [`ActiveEventLoop`](winit::event_loop::ActiveEventLoop)
/// callback, whereas this port is pull-based — `create_window` must return a
/// [`WindowId`] synchronously. The backend bridges the two by *deferring*: it
/// mints the id immediately and queues the request, and the
/// [`WinitAppHandler`] materializes the OS window on the next event-loop turn.
/// The port contract is honored (a valid id is returned); the window simply
/// appears one turn later.
///
/// # State split: owned records vs. shared winit state
///
/// The port's `window()` returns `&Window`, which cannot borrow through a mutex
/// guard. So the retained domain [`Window`] records — and the id counter — are
/// owned **directly** by the backend, giving `window()` a plain borrow. Only the
/// genuinely winit-side, handler-shared state (pending work queues, live winit
/// windows, the id maps, the event buffer) lives behind
/// `Arc<Mutex<BackendState>>`. Mutating operations update the owned record *and*
/// enqueue the corresponding winit work for the handler to perform.
///
/// # Wiring a host into an event loop
///
/// ```no_run
/// use obscur_window_winit::backend::WinitWindowBackend;
/// use obscur_window_winit::backend::UserEvent;
/// use obscur_window_winit::wake::EventLoopWaker;
/// use winit::event_loop::EventLoop;
///
/// let mut backend = WinitWindowBackend::new();
/// // The backend can be driven (create_window, update_window, ...) before or
/// // during the run; queued work is applied on each loop turn.
///
/// let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
/// // A cross-thread frame-scheduling waker, if desired:
/// let _waker = EventLoopWaker::new(event_loop.create_proxy());
///
/// // The handler shares the backend's state, so operations issued on `backend`
/// // are observed by the running loop.
/// let mut handler = backend.app_handler();
/// event_loop.run_app(&mut handler).unwrap();
/// ```
#[derive(Debug)]
pub struct WinitWindowBackend {
    /// The source of fresh window identities: a plain monotonic counter, owned
    /// by the backend so ids can be minted without touching the shared state.
    next_id: u64,
    /// The retained domain records, owned directly so `window()` can borrow one
    /// without going through the mutex.
    records: HashMap<WindowId, Window>,
    /// The winit-side state shared with the [`WinitAppHandler`].
    state: Arc<Mutex<BackendState>>,
}

impl WinitWindowBackend {
    /// Creates an empty backend with no windows and no queued work.
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_id: 0,
            records: HashMap::new(),
            state: Arc::new(Mutex::new(BackendState::default())),
        }
    }

    /// Builds a [`WinitAppHandler`] sharing this backend's winit-side state.
    ///
    /// Hand the returned handler to
    /// [`run_app`](winit::event_loop::EventLoop::run_app); it observes and
    /// services every operation issued on this backend. See the type-level docs
    /// for a full wiring example.
    #[must_use]
    pub fn app_handler(&self) -> WinitAppHandler {
        WinitAppHandler::new(Arc::clone(&self.state))
    }

    /// Mints the next fresh [`WindowId`] from the owned monotonic counter.
    fn allocate_id(&mut self) -> WindowId {
        let id = WindowId::from_raw(self.next_id);
        self.next_id += 1;
        id
    }

    /// Locks the shared winit-side state, treating poisoning as a bug.
    fn lock(&self) -> std::sync::MutexGuard<'_, BackendState> {
        self.state.lock().expect("backend state mutex poisoned")
    }

    /// Queues the current desired properties of `id` for the handler to push
    /// onto the live winit window on the next turn.
    fn queue_property_push(&self, id: WindowId) {
        if let Some(record) = self.records.get(&id) {
            let properties = record.properties().clone();
            self.lock().pending_property_updates.push((id, properties));
        }
    }
}

impl Default for WinitWindowBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowBackend for WinitWindowBackend {
    fn create_window(&mut self, request: CreateWindow) -> Result<WindowId, WindowError> {
        // Deferred creation: mint the id and record now, materialize later.
        let id = self.allocate_id();
        self.records.insert(id, request.clone().into_window(id));
        self.lock().pending_creates.push((id, request));
        Ok(id)
    }

    fn close_window(&mut self, request: CloseWindow) -> WindowOperationResult {
        if self.records.remove(&request.target()).is_some() {
            self.lock().pending_closes.push(request.target());
            WindowOperationResult::native()
        } else {
            WindowOperationResult::failed("unknown window")
        }
    }

    fn window(&self, id: WindowId) -> Option<&Window> {
        self.records.get(&id)
    }

    fn update_window(&mut self, changes: &UpdateWindow) -> WindowOperationResult {
        match self.records.get_mut(&changes.target()) {
            Some(record) => {
                changes.apply(record.properties_mut());
                self.queue_property_push(changes.target());
                WindowOperationResult::native()
            }
            None => WindowOperationResult::failed("unknown window"),
        }
    }

    fn activate(&mut self, request: ActivateWindow) -> ActivationResult {
        // Focus is granted by the handler on the live window; element focus is
        // out of scope here, so any FocusRequest activates the window only.
        if self.records.contains_key(&request.target()) {
            self.lock().pending_focus.push(request.target());
            ActivationResult::Activated
        } else {
            ActivationResult::Failed(Box::from("unknown window"))
        }
    }

    fn set_chrome_mode(&mut self, id: WindowId, mode: ChromeMode) -> WindowOperationResult {
        // winit can toggle server-side decorations: client-side/custom chrome
        // means "no server decorations". Custom-chrome drag/resize *regions* are
        // separately unsupported (see set_drag_regions/set_resize_regions).
        match self.records.get_mut(&id) {
            Some(record) => {
                record.properties_mut().decorated = !mode.uses_client_side();
                self.queue_property_push(id);
                WindowOperationResult::native()
            }
            None => WindowOperationResult::failed("unknown window"),
        }
    }

    fn set_drag_regions(
        &mut self,
        _id: WindowId,
        _regions: &[DragRegion],
    ) -> WindowOperationResult {
        // winit exposes only whole-window drag, not app-declared drag regions.
        WindowOperationResult::Unsupported
    }

    fn set_resize_regions(
        &mut self,
        _id: WindowId,
        _regions: &[ResizeRegion],
    ) -> WindowOperationResult {
        // winit exposes only edge resize gestures, not app-declared regions.
        WindowOperationResult::Unsupported
    }

    fn show_system_menu(
        &mut self,
        _id: WindowId,
        _request: SystemMenuRequest,
    ) -> WindowOperationResult {
        // No portable system-menu invocation is wired up.
        WindowOperationResult::Unsupported
    }

    fn set_presentation(
        &mut self,
        id: WindowId,
        mode: &PresentationMode,
    ) -> WindowOperationResult {
        // Fullscreen is real: the handler applies borderless fullscreen via the
        // window's state. Exclusive/kiosk forms degrade to borderless, which is
        // reflected honestly by supports(ExclusiveFullscreen)/supports(Kiosk).
        match self.records.get_mut(&id) {
            Some(record) => {
                record.properties_mut().state = if mode.fullscreen.is_fullscreen() {
                    WindowState::Fullscreen
                } else {
                    WindowState::Normal
                };
                self.queue_property_push(id);
                WindowOperationResult::native()
            }
            None => WindowOperationResult::failed("unknown window"),
        }
    }

    fn inhibit_sleep(&mut self, _request: SleepInhibition) -> WindowOperationResult {
        // No portable sleep/screensaver inhibition is wired up.
        WindowOperationResult::Unsupported
    }

    fn displays(&self) -> Vec<Display> {
        // Monitor enumeration needs an ActiveEventLoop; a host obtains displays
        // through the event loop directly rather than through this snapshot.
        Vec::new()
    }

    fn primary_display(&self) -> Option<DisplayId> {
        // See displays(): no monitor handle is reachable without the event loop.
        None
    }

    #[allow(clippy::match_same_arms)] // Each feature is documented distinctly.
    fn supports(&self, feature: WindowFeature) -> SupportLevel {
        match feature {
            // Set natively by winit at creation / via window level.
            WindowFeature::Transparency
            | WindowFeature::AlwaysOnTop
            | WindowFeature::PerMonitorDpi => SupportLevel::Native,
            // Server decorations toggle natively, but app-declared drag/resize
            // regions are emulated at best.
            WindowFeature::CustomChrome => SupportLevel::Emulated,
            // Blur/backdrop is platform-conditional in winit (macOS/Windows).
            WindowFeature::BlurBehind => SupportLevel::Limited,
            // This backend degrades exclusive fullscreen to borderless.
            WindowFeature::ExclusiveFullscreen => SupportLevel::Limited,
            // Wayland hides global screen coordinates; reported honestly.
            WindowFeature::GlobalDisplayCoordinates => SupportLevel::Limited,
            // No kiosk lock-down, system menu, or sleep inhibition is wired up.
            WindowFeature::Kiosk
            | WindowFeature::SystemMenu
            | WindowFeature::SleepInhibition => SupportLevel::Unavailable,
        }
    }

    fn poll_window_events(&mut self) -> Vec<(WindowId, WindowEvent)> {
        std::mem::take(&mut self.lock().window_events)
    }

    fn poll_display_events(&mut self) -> Vec<DisplayEvent> {
        // Display hotplug events require monitor tracking through the event loop,
        // which this snapshot backend does not perform.
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Size;
    use obscur_windowing::window::WindowProperties;

    fn backend() -> WinitWindowBackend {
        WinitWindowBackend::new()
    }

    #[test]
    fn create_returns_incrementing_ids_and_retains_records() {
        let mut backend = backend();
        let first = backend
            .create_window(CreateWindow::new(WindowProperties::default().with_title("Main")))
            .expect("create always succeeds (deferred)");
        let second = backend
            .create_window(CreateWindow::new(WindowProperties::default()))
            .expect("create always succeeds (deferred)");

        assert_ne!(first, second);
        assert_eq!(first, WindowId::from_raw(0));
        assert_eq!(second, WindowId::from_raw(1));

        let record = backend.window(first).expect("record retained");
        assert_eq!(record.id(), first);
        assert_eq!(&*record.properties().title, "Main");
        // The create request was queued for the handler to materialize.
        assert_eq!(backend.lock().pending_creates.len(), 2);
    }

    #[test]
    fn close_known_window_succeeds_and_forgets_it() {
        let mut backend = backend();
        let id = backend
            .create_window(CreateWindow::new(WindowProperties::default()))
            .unwrap();

        assert_eq!(
            backend.close_window(CloseWindow::request(id)),
            WindowOperationResult::native()
        );
        assert!(backend.window(id).is_none());
        assert_eq!(backend.lock().pending_closes, vec![id]);
    }

    #[test]
    fn close_unknown_window_fails() {
        let mut backend = backend();
        assert_eq!(
            backend.close_window(CloseWindow::request(WindowId::from_raw(42))),
            WindowOperationResult::failed("unknown window")
        );
    }

    #[test]
    fn update_mutates_the_record_and_queues_a_push() {
        let mut backend = backend();
        let id = backend
            .create_window(CreateWindow::new(WindowProperties::default().with_title("Old")))
            .unwrap();

        let delta = UpdateWindow::new(id)
            .set_title("New")
            .set_size(Size::new(320.0, 240.0));
        assert_eq!(backend.update_window(&delta), WindowOperationResult::native());

        let record = backend.window(id).expect("record retained");
        assert_eq!(&*record.properties().title, "New");
        assert_eq!(record.properties().size, Size::new(320.0, 240.0));

        let state = backend.lock();
        assert_eq!(state.pending_property_updates.len(), 1);
        assert_eq!(state.pending_property_updates[0].0, id);
        assert_eq!(&*state.pending_property_updates[0].1.title, "New");
    }

    #[test]
    fn update_unknown_window_fails() {
        let mut backend = backend();
        assert_eq!(
            backend.update_window(&UpdateWindow::new(WindowId::from_raw(7)).set_resizable(false)),
            WindowOperationResult::failed("unknown window")
        );
    }

    #[test]
    fn activate_known_window_is_activated_and_queues_focus() {
        let mut backend = backend();
        let id = backend
            .create_window(CreateWindow::new(WindowProperties::default()))
            .unwrap();
        assert_eq!(
            backend.activate(ActivateWindow::window(id)),
            ActivationResult::Activated
        );
        assert_eq!(backend.lock().pending_focus, vec![id]);
    }

    #[test]
    fn activate_unknown_window_fails() {
        let mut backend = backend();
        assert_eq!(
            backend.activate(ActivateWindow::window(WindowId::from_raw(3))),
            ActivationResult::Failed(Box::from("unknown window"))
        );
    }

    #[test]
    fn chrome_mode_toggles_decorations_on_the_record() {
        let mut backend = backend();
        let id = backend
            .create_window(CreateWindow::new(WindowProperties::default()))
            .unwrap();
        assert!(backend.window(id).unwrap().properties().decorated);

        assert_eq!(
            backend.set_chrome_mode(id, ChromeMode::Custom),
            WindowOperationResult::native()
        );
        assert!(!backend.window(id).unwrap().properties().decorated);
    }

    #[test]
    fn presentation_fullscreen_sets_state_and_is_native() {
        let mut backend = backend();
        let id = backend
            .create_window(CreateWindow::new(WindowProperties::default()))
            .unwrap();
        let mode = PresentationMode::new()
            .with_fullscreen(obscur_windowing::presentation::FullscreenMode::Borderless);
        assert_eq!(
            backend.set_presentation(id, &mode),
            WindowOperationResult::native()
        );
        assert_eq!(
            backend.window(id).unwrap().properties().state,
            WindowState::Fullscreen
        );
    }

    #[test]
    fn unsupported_operations_report_unsupported() {
        let mut backend = backend();
        let id = WindowId::from_raw(0);
        assert_eq!(
            backend.set_drag_regions(id, &[]),
            WindowOperationResult::Unsupported
        );
        assert_eq!(
            backend.set_resize_regions(id, &[]),
            WindowOperationResult::Unsupported
        );
        assert_eq!(
            backend.inhibit_sleep(SleepInhibition::reasonless(
                obscur_windowing::presentation::SleepInhibitionScope::System
            )),
            WindowOperationResult::Unsupported
        );
    }

    #[test]
    fn supports_reports_documented_levels() {
        let backend = backend();
        assert_eq!(
            backend.supports(WindowFeature::Transparency),
            SupportLevel::Native
        );
        assert_eq!(
            backend.supports(WindowFeature::AlwaysOnTop),
            SupportLevel::Native
        );
        assert_eq!(
            backend.supports(WindowFeature::PerMonitorDpi),
            SupportLevel::Native
        );
        assert_eq!(
            backend.supports(WindowFeature::CustomChrome),
            SupportLevel::Emulated
        );
        assert_eq!(
            backend.supports(WindowFeature::BlurBehind),
            SupportLevel::Limited
        );
        assert_eq!(
            backend.supports(WindowFeature::GlobalDisplayCoordinates),
            SupportLevel::Limited
        );
        assert_eq!(
            backend.supports(WindowFeature::Kiosk),
            SupportLevel::Unavailable
        );
        assert_eq!(
            backend.supports(WindowFeature::SleepInhibition),
            SupportLevel::Unavailable
        );
    }

    #[test]
    fn displays_and_display_events_are_empty() {
        let mut backend = backend();
        assert!(backend.displays().is_empty());
        assert_eq!(backend.primary_display(), None);
        assert!(backend.poll_display_events().is_empty());
    }

    #[test]
    fn poll_window_events_drains_the_injected_buffer() {
        let mut backend = backend();
        assert!(backend.poll_window_events().is_empty());

        // Manually inject events as the handler would, without an event loop.
        let id = WindowId::from_raw(0);
        backend
            .lock()
            .window_events
            .push((id, WindowEvent::CloseRequested));
        backend
            .lock()
            .window_events
            .push((id, WindowEvent::FocusGained));

        let drained = backend.poll_window_events();
        assert_eq!(
            drained,
            vec![
                (id, WindowEvent::CloseRequested),
                (id, WindowEvent::FocusGained)
            ]
        );
        // A second poll is empty: the buffer was taken.
        assert!(backend.poll_window_events().is_empty());
    }

    #[test]
    fn is_object_safe() {
        let backend: Box<dyn WindowBackend> = Box::new(WinitWindowBackend::new());
        assert!(backend.displays().is_empty());
    }
}
