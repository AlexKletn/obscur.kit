//! The [`WinitWindowBackend`] winit adapter implementing the [`WindowBackend`]
//! port (spec §6.1-6.6 windowing; §2.7 "Реализует `WindowBackend`").

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use obscur_capability::SupportLevel;
use obscur_identity::WindowId;
use obscur_windowing::chrome::{ChromeMode, DragRegion, ResizeRegion, SystemMenuRequest};
use obscur_windowing::display::{Display, DisplayEvent, DisplayId};
use obscur_windowing::port::{WindowBackend, WindowError, WindowFeature};
use obscur_windowing::presentation::{
    FullscreenMode, PresentationMode, SafeEscapePolicy, SleepInhibition,
};
use obscur_windowing::request::{
    ActivateWindow, ActivationResult, CloseWindow, CreateWindow, UpdateWindow,
    WindowOperationResult,
};
use obscur_windowing::window::{Window, WindowEvent, WindowRelation, WindowState};

use super::backend_state::BackendState;
use super::winit_event_loop::WinitAppHandler;
use super::winit_window::WinitWindow;

#[doc = include_str!("../../docs/api/backend/winit_window_backend.doc/WinitWindowBackend.md")]
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

    #[doc = include_str!("../../docs/api/backend/winit_window_backend.doc/app_handler.md")]
    #[must_use]
    pub fn app_handler(&self) -> WinitAppHandler {
        WinitAppHandler::new(Arc::clone(&self.state))
    }

    #[doc = include_str!("../../docs/api/backend/winit_window_backend.doc/native_window.md")]
    #[must_use]
    pub fn native_window(&self, id: WindowId) -> Option<Arc<winit::window::Window>> {
        self.lock().windows.get(&id).map(WinitWindow::shared)
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
        // Fail closed rather than hand back an id for a window that would ignore
        // half of what was asked for.
        if let Some(reason) = unsatisfiable_relation(request.relation()) {
            return Err(WindowError::InvalidRequest(reason));
        }
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
        let target = changes.target();
        let Some(record) = self.records.get_mut(&target) else {
            return WindowOperationResult::failed("unknown window");
        };

        let was_transparent = record.properties().transparent;
        changes.apply(record.properties_mut());
        // winit fixes transparency at creation and offers no setter, so a
        // requested flip is dropped. The record is rolled back to the live value
        // so `window()` keeps describing the window that actually exists, and the
        // caller is told the update was only conditionally satisfied.
        let transparency_dropped = record.properties().transparent != was_transparent;
        record.properties_mut().transparent = was_transparent;

        self.queue_property_push(target);
        if transparency_dropped {
            WindowOperationResult::conditional()
        } else {
            WindowOperationResult::native()
        }
    }

    fn activate(&mut self, request: ActivateWindow) -> ActivationResult {
        let target = request.target();
        if !self.records.contains_key(&target) {
            return ActivationResult::Failed(Box::from("unknown window"));
        }
        // Element focus belongs to the runtime's focus tree, not to a window
        // backend: there is nothing here that could move it.
        if request.focus().element().is_some() {
            return ActivationResult::Unsupported;
        }

        let mut state = self.lock();
        // The compositor's own verdict, recorded from native `Focused` events, is
        // the one thing that can honestly justify `Activated`.
        if state.focused == Some(target) {
            return ActivationResult::Activated;
        }
        state.pending_focus.push(target);
        // Everything else is `Denied` — the §6.3 normal outcome, not a failure.
        // The request is queued for the event loop, but winit grants focus
        // asynchronously and the compositor may refuse it outright (it always
        // does on Wayland, where `focus_window` is a no-op), so at this instant
        // the window is provably *not* activated. A later grant arrives as
        // `WindowEvent::FocusGained` from `poll_window_events`, which is how a
        // caller learns activation eventually happened.
        ActivationResult::Denied
    }

    fn set_chrome_mode(&mut self, id: WindowId, mode: ChromeMode) -> WindowOperationResult {
        // winit can toggle server-side decorations: client-side/custom chrome
        // means "no server decorations". That is only half of §6.4 — the drag /
        // resize regions and the title bar itself are the kit's to draw, and the
        // region setters below are unsupported — so client-side modes report
        // `emulated`, agreeing with supports(CustomChrome).
        match self.records.get_mut(&id) {
            Some(record) => {
                record.properties_mut().decorated = !mode.uses_client_side();
                self.queue_property_push(id);
                if mode.uses_client_side() {
                    WindowOperationResult::emulated()
                } else {
                    WindowOperationResult::native()
                }
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

    fn set_presentation(&mut self, id: WindowId, mode: &PresentationMode) -> WindowOperationResult {
        if !self.records.contains_key(&id) {
            return WindowOperationResult::failed("unknown window");
        }
        // Borderless is the only fullscreen form this backend has. Kiosk asks for
        // a lock-down it cannot even approximate, so it is refused outright
        // instead of silently presenting an escapable borderless window.
        let outcome = presentation_outcome(mode);
        if outcome.is_unsupported() {
            return outcome;
        }

        let record = self.records.get_mut(&id).expect("membership checked above");
        record.properties_mut().state = if mode.fullscreen.is_fullscreen() {
            WindowState::Fullscreen
        } else {
            WindowState::Normal
        };
        self.queue_property_push(id);
        outcome
    }

    fn inhibit_sleep(&mut self, _request: SleepInhibition) -> WindowOperationResult {
        // No portable sleep/screensaver inhibition is wired up.
        WindowOperationResult::Unsupported
    }

    fn displays(&self) -> Vec<Display> {
        // Monitor handles are only reachable through an ActiveEventLoop, so the
        // handler re-enumerates them on each turn and this reads what it left,
        // exactly as `window()` reads a record the handler materialized. Empty
        // until the first turn, which is when the topology first exists to know.
        self.lock().displays.clone()
    }

    fn primary_display(&self) -> Option<DisplayId> {
        self.lock()
            .displays
            .iter()
            .find(|display| display.is_primary())
            .map(Display::id)
    }

    #[allow(clippy::match_same_arms)] // Each feature is documented distinctly.
    fn supports(&self, feature: WindowFeature) -> SupportLevel {
        match feature {
            // Set natively by winit at creation / via window level.
            WindowFeature::AlwaysOnTop | WindowFeature::PerMonitorDpi => SupportLevel::Native,
            // Honored at creation only: winit has no post-creation setter, so
            // update_window reports a requested flip as conditional.
            WindowFeature::Transparency => SupportLevel::Limited,
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
            WindowFeature::Kiosk | WindowFeature::SystemMenu | WindowFeature::SleepInhibition => {
                SupportLevel::Unavailable
            }
        }
    }

    fn poll_window_events(&mut self) -> Vec<(WindowId, WindowEvent)> {
        std::mem::take(&mut self.lock().window_events)
    }

    fn poll_display_events(&mut self) -> Vec<DisplayEvent> {
        std::mem::take(&mut self.lock().display_events)
    }
}

#[doc = include_str!("../../docs/api/backend/winit_window_backend.doc/unsatisfiable_relation.md")]
fn unsatisfiable_relation(relation: WindowRelation) -> Option<Box<str>> {
    if relation.owner.is_some() {
        return Some(Box::from(
            "winit has no owner/parent window relation: it offers only clipped child windows",
        ));
    }
    if relation.modality.is_modal() {
        return Some(Box::from(
            "winit cannot make a window modal: it has no input-blocking primitive",
        ));
    }
    None
}

#[doc = include_str!("../../docs/api/backend/winit_window_backend.doc/presentation_outcome.md")]
fn presentation_outcome(mode: &PresentationMode) -> WindowOperationResult {
    match mode.fullscreen {
        FullscreenMode::Kiosk => WindowOperationResult::Unsupported,
        FullscreenMode::Exclusive | FullscreenMode::Presentation => {
            WindowOperationResult::conditional()
        }
        FullscreenMode::Normal | FullscreenMode::Borderless => {
            if drops_presentation_controls(mode) {
                WindowOperationResult::conditional()
            } else {
                WindowOperationResult::native()
            }
        }
    }
}

/// Returns whether `mode` asks for a §6.5 optional control this backend drops.
fn drops_presentation_controls(mode: &PresentationMode) -> bool {
    mode.hide_cursor
        || mode.inhibit_sleep
        || mode.target_display.is_some()
        || mode.safe_escape != SafeEscapePolicy::Allow
}

#[doc = include_str!("../../docs/api/backend/winit_window_backend.doc/live_tests.md")]
#[cfg(all(test, target_os = "linux"))]
mod live_tests {
    use super::*;
    use obscur_windowing::window::WindowProperties;
    use winit::application::ApplicationHandler;
    use winit::event::WindowEvent as WinitWindowEvent;
    use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
    use winit::window::WindowId as WinitWindowId;

    use super::super::{UserEvent, WinitAppHandler};

    #[doc = include_str!("../../docs/api/backend/winit_window_backend.doc/display_event_loop.md")]
    fn display_event_loop() -> Option<EventLoop<UserEvent>> {
        use winit::platform::x11::EventLoopBuilderExtX11;

        match EventLoop::<UserEvent>::with_user_event()
            .with_any_thread(true)
            .build()
        {
            Ok(event_loop) => Some(event_loop),
            // Print, never swallow. A silent `.ok()` here is how this test passes
            // while proving nothing — the reason it skipped must reach the log so
            // "0 windows opened" can be told apart from "the path is fine".
            Err(error) => {
                eprintln!("[live] no event loop, skipping: {error}");
                None
            }
        }
    }

    #[doc = include_str!("../../docs/api/backend/winit_window_backend.doc/Observations.md")]
    #[derive(Debug, Default)]
    struct Observations {
        /// Whether each window's platform handle was reachable once pumped.
        handles_after_pump: Option<(bool, bool)>,
        /// Whether each id's handle is *its own* window — the window whose native
        /// events route back to that same id.
        each_handle_routes_to_its_own_id: Option<(bool, bool)>,
        /// Whether the port reported both materializations to the host.
        created_events: Option<bool>,
        /// Whether the closed window's handle went, and the other one stayed.
        handles_after_closing_the_first: Option<(bool, bool)>,
    }

    #[doc = include_str!("../../docs/api/backend/winit_window_backend.doc/LiveProbe.md")]
    struct LiveProbe {
        /// The backend's handler, forwarded to on every callback.
        inner: WinitAppHandler,
        /// The port handle the host issues its operations on.
        backend: WinitWindowBackend,
        /// The two windows this probe created through the port. Two, not one,
        /// because §6.1 is a *multi-window* API and because one window cannot
        /// tell "the handle for this id" apart from "a handle".
        ids: (WindowId, WindowId),
        /// Turns of `about_to_wait` seen so far.
        turn: u32,
        /// What was observed, read by the test after the loop returns.
        observed: Observations,
    }

    impl LiveProbe {
        /// Returns whether the live handle for `id` is the window whose native
        /// events route back to `id`.
        fn handle_routes_to_own_id(&self, id: WindowId) -> bool {
            self.backend.native_window(id).is_some_and(|window| {
                self.backend.lock().winit_to_domain.get(&window.id()) == Some(&id)
            })
        }
    }

    impl ApplicationHandler<UserEvent> for LiveProbe {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            self.inner.resumed(event_loop);
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            id: WinitWindowId,
            event: WinitWindowEvent,
        ) {
            self.inner.window_event(event_loop, id, event);
        }

        fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
            // Poll, not Wait: the close issued on turn 0 is serviced by turn 1's
            // pump, and under `Wait` that turn would never come on an idle
            // compositor — the loop would block for an input event that is not
            // coming and the test would hang.
            event_loop.set_control_flow(ControlFlow::Poll);
            self.inner.about_to_wait(event_loop);
            let (first, second) = self.ids;

            if self.turn == 0 {
                self.observed.handles_after_pump = Some((
                    self.backend.native_window(first).is_some(),
                    self.backend.native_window(second).is_some(),
                ));
                self.observed.each_handle_routes_to_its_own_id = Some((
                    self.handle_routes_to_own_id(first),
                    self.handle_routes_to_own_id(second),
                ));
                let events = self.backend.poll_window_events();
                self.observed.created_events = Some(
                    events.contains(&(first, WindowEvent::Created))
                        && events.contains(&(second, WindowEvent::Created)),
                );
                // Close only the first; the next turn's pump performs it.
                self.backend.close_window(CloseWindow::request(first));
            } else {
                self.observed.handles_after_closing_the_first = Some((
                    self.backend.native_window(first).is_some(),
                    self.backend.native_window(second).is_some(),
                ));
                event_loop.exit();
            }
            self.turn += 1;
        }
    }

    #[doc = include_str!("../../docs/api/backend/winit_window_backend.doc/a_host_reaches_its_own_real_windows_through_the_port.md")]
    #[test]
    #[ignore = "opens real OS windows: attended proof only, and vacuous headless — see the module docs"]
    fn a_host_reaches_its_own_real_windows_through_the_port() {
        let Some(event_loop) = display_event_loop() else {
            return; // Headless: nothing to open a window on. See the module docs.
        };

        let mut backend = WinitWindowBackend::new();
        let mut create = |title: &str| {
            backend
                .create_window(CreateWindow::new(
                    WindowProperties::default().with_title(title),
                ))
                .expect("a top-level modeless window is satisfiable")
        };
        let ids = (create("port probe A"), create("port probe B"));

        // Deferred creation: both ids are already valid and neither OS window is
        // there. This is the state a host sees between `create_window` and the
        // first loop turn, and it must be an honest `None`.
        assert!(backend.native_window(ids.0).is_none());
        assert!(backend.native_window(ids.1).is_none());

        let mut probe = LiveProbe {
            inner: backend.app_handler(),
            backend,
            ids,
            turn: 0,
            observed: Observations::default(),
        };
        event_loop.run_app(&mut probe).expect("the loop runs");

        let observed = probe.observed;
        assert_eq!(
            observed.handles_after_pump,
            Some((true, true)),
            "the handler materializes every queued window on its first turn, and \
             the host must be able to reach them: this is what each windowed \
             example calls `event_loop.create_window` itself to get"
        );
        assert_eq!(
            observed.each_handle_routes_to_its_own_id,
            Some((true, true)),
            "each handle handed to the host must be the window whose native events \
             route back to the id it was asked for"
        );
        assert_eq!(
            observed.created_events,
            Some(true),
            "the port must report the materializations the host waits on"
        );
        assert_eq!(
            observed.handles_after_closing_the_first,
            Some((false, true)),
            "a closed window's handle must not outlive the close, and closing one \
             window must not take its sibling with it"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Size;
    use obscur_identity::NodeId;
    use obscur_windowing::window::{Modality, WindowProperties};

    fn backend() -> WinitWindowBackend {
        WinitWindowBackend::new()
    }

    #[test]
    fn create_returns_incrementing_ids_and_retains_records() {
        let mut backend = backend();
        let first = backend
            .create_window(CreateWindow::new(
                WindowProperties::default().with_title("Main"),
            ))
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

    /// The deferred window, stated as a contract: between `create_window` and the
    /// handler's first turn the id is valid and no OS window stands behind it, so
    /// the platform handle is honestly absent rather than fabricated. An unknown
    /// id is the same absence.
    ///
    /// This is the half of `native_window` that is decidable without a display,
    /// so it is the half that runs in CI; the `Some` half needs a real window and
    /// lives in `live_tests`, which is inert headlessly.
    #[test]
    fn native_window_is_none_until_the_handler_pumps() {
        let mut backend = backend();
        let id = backend
            .create_window(CreateWindow::new(WindowProperties::default()))
            .expect("create always succeeds (deferred)");

        assert!(backend.window(id).is_some(), "the domain record exists now");
        assert!(
            backend.native_window(id).is_none(),
            "but no OS window does until the handler pumps"
        );
        assert!(backend.native_window(WindowId::from_raw(404)).is_none());
    }

    /// §6.1: winit has no owner relation and no modality. Returning an id here
    /// would promise an owned/modal window and deliver a top-level modeless one,
    /// so the request is refused with its reason instead.
    #[test]
    fn create_rejects_relations_winit_cannot_materialize() {
        let mut backend = backend();

        let owned = CreateWindow::new(WindowProperties::default())
            .with_relation(WindowRelation::owned_by(WindowId::from_raw(0)));
        let owned_error = backend.create_window(owned).expect_err("owner is refused");
        assert!(matches!(owned_error, WindowError::InvalidRequest(_)));

        let modal = CreateWindow::new(WindowProperties::default())
            .with_relation(WindowRelation::top_level().with_modality(Modality::ApplicationModal));
        let modal_error = backend
            .create_window(modal)
            .expect_err("modality is refused");
        assert!(matches!(modal_error, WindowError::InvalidRequest(_)));

        // Nothing was minted or queued for either refusal.
        assert!(backend.window(WindowId::from_raw(0)).is_none());
        assert!(backend.lock().pending_creates.is_empty());

        // The ordinary top-level, modeless request is untouched.
        assert!(
            backend
                .create_window(CreateWindow::new(WindowProperties::default()))
                .is_ok()
        );
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
            .create_window(CreateWindow::new(
                WindowProperties::default().with_title("Old"),
            ))
            .unwrap();

        let delta = UpdateWindow::new(id)
            .set_title("New")
            .set_size(Size::new(320.0, 240.0));
        assert_eq!(
            backend.update_window(&delta),
            WindowOperationResult::native()
        );

        let record = backend.window(id).expect("record retained");
        assert_eq!(&*record.properties().title, "New");
        assert_eq!(record.properties().size, Size::new(320.0, 240.0));

        let state = backend.lock();
        assert_eq!(state.pending_property_updates.len(), 1);
        assert_eq!(state.pending_property_updates[0].0, id);
        assert_eq!(&*state.pending_property_updates[0].1.title, "New");
    }

    /// §6.2: winit fixes transparency at creation, so a requested flip is
    /// dropped — the update is conditional, and the record must keep describing
    /// the window that actually exists rather than the one that was asked for.
    #[test]
    fn update_transparency_is_conditional_and_does_not_forge_the_record() {
        let mut backend = backend();
        let id = backend
            .create_window(CreateWindow::new(WindowProperties::default()))
            .unwrap();
        assert!(!backend.window(id).unwrap().properties().transparent);

        let delta = UpdateWindow::new(id).set_transparent(true).set_title("T");
        assert_eq!(
            backend.update_window(&delta),
            WindowOperationResult::conditional()
        );
        // The flip did not happen, so the record must not claim it did...
        assert!(!backend.window(id).unwrap().properties().transparent);
        // ...while everything winit *can* set was still applied.
        assert_eq!(&*backend.window(id).unwrap().properties().title, "T");
    }

    /// A no-op transparency request is not a dropped one: an update that leaves
    /// the flag where it already is stays native.
    #[test]
    fn update_transparency_to_its_current_value_stays_native() {
        let mut backend = backend();
        let id = backend
            .create_window(CreateWindow::new(WindowProperties::default()))
            .unwrap();
        assert_eq!(
            backend.update_window(&UpdateWindow::new(id).set_transparent(false)),
            WindowOperationResult::native()
        );
    }

    #[test]
    fn update_unknown_window_fails() {
        let mut backend = backend();
        assert_eq!(
            backend.update_window(&UpdateWindow::new(WindowId::from_raw(7)).set_resizable(false)),
            WindowOperationResult::failed("unknown window")
        );
    }

    /// §6.3: an activation nobody has observed is `Denied`, not `Activated`.
    /// The queued focus request is still submitted for the event loop.
    #[test]
    fn activate_without_an_observed_grant_is_denied_and_queues_focus() {
        let mut backend = backend();
        let id = backend
            .create_window(CreateWindow::new(WindowProperties::default()))
            .unwrap();
        assert_eq!(
            backend.activate(ActivateWindow::window(id)),
            ActivationResult::Denied
        );
        assert_eq!(backend.lock().pending_focus, vec![id]);
    }

    /// §6.3: `Activated` is only reported once the compositor has said so via a
    /// native `Focused` event, which the handler records as `focused`.
    #[test]
    fn activate_reports_activated_only_once_the_compositor_granted_focus() {
        let mut backend = backend();
        let id = backend
            .create_window(CreateWindow::new(WindowProperties::default()))
            .unwrap();

        backend.lock().focused = Some(id);
        assert_eq!(
            backend.activate(ActivateWindow::window(id)),
            ActivationResult::Activated
        );
        // Already focused: nothing needed queueing.
        assert!(backend.lock().pending_focus.is_empty());

        // Focus held elsewhere is not this window's activation.
        backend.lock().focused = Some(WindowId::from_raw(99));
        assert_eq!(
            backend.activate(ActivateWindow::window(id)),
            ActivationResult::Denied
        );
    }

    /// §6.3: this backend has no element-focus mechanism, so it says so instead
    /// of activating the window and calling that a focused element.
    #[test]
    fn activate_element_focus_is_unsupported() {
        let mut backend = backend();
        let id = backend
            .create_window(CreateWindow::new(WindowProperties::default()))
            .unwrap();
        assert_eq!(
            backend.activate(ActivateWindow::element(id, NodeId::from_raw(1))),
            ActivationResult::Unsupported
        );
        assert!(backend.lock().pending_focus.is_empty());
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
            WindowOperationResult::emulated()
        );
        assert!(!backend.window(id).unwrap().properties().decorated);
    }

    /// §6.4/§6.2: the result and the capability must agree — client-side chrome
    /// is `Emulated` in `supports`, so it cannot be `native()` here.
    #[test]
    fn chrome_mode_result_agrees_with_the_reported_capability() {
        let mut backend = backend();
        let id = backend
            .create_window(CreateWindow::new(WindowProperties::default()))
            .unwrap();

        for mode in [ChromeMode::ClientSide, ChromeMode::Custom] {
            assert_eq!(
                backend.set_chrome_mode(id, mode).support_level(),
                Some(backend_support_level(WindowFeature::CustomChrome)),
                "{mode:?} must report the level supports(CustomChrome) advertises"
            );
        }
        // The system frame is winit's own: that one really is native.
        assert_eq!(
            backend.set_chrome_mode(id, ChromeMode::SystemFrame),
            WindowOperationResult::native()
        );
    }

    fn backend_support_level(feature: WindowFeature) -> SupportLevel {
        backend().supports(feature)
    }

    #[test]
    fn presentation_borderless_sets_state_and_is_native() {
        let mut backend = backend();
        let id = backend
            .create_window(CreateWindow::new(WindowProperties::default()))
            .unwrap();
        let mode = PresentationMode::new().with_fullscreen(FullscreenMode::Borderless);
        assert_eq!(
            backend.set_presentation(id, &mode),
            WindowOperationResult::native()
        );
        assert_eq!(
            backend.window(id).unwrap().properties().state,
            WindowState::Fullscreen
        );
    }

    /// §6.5: exclusive and presentation forms are served by borderless, so they
    /// are conditional — never `native()`.
    #[test]
    fn presentation_degraded_forms_are_conditional() {
        let mut backend = backend();
        let id = backend
            .create_window(CreateWindow::new(WindowProperties::default()))
            .unwrap();

        for form in [FullscreenMode::Exclusive, FullscreenMode::Presentation] {
            let mode = PresentationMode::new().with_fullscreen(form);
            assert_eq!(
                backend.set_presentation(id, &mode),
                WindowOperationResult::conditional(),
                "{form:?} collapses to borderless and must say so"
            );
            // It is still performed, degraded: the window does go fullscreen.
            assert_eq!(
                backend.window(id).unwrap().properties().state,
                WindowState::Fullscreen
            );
        }
    }

    /// §6.5: kiosk has no lock-down here, and supports(Kiosk) is `Unavailable`.
    /// It must be refused, not silently served as an escapable borderless window.
    #[test]
    fn presentation_kiosk_is_unsupported_and_changes_nothing() {
        let mut backend = backend();
        let id = backend
            .create_window(CreateWindow::new(WindowProperties::default()))
            .unwrap();
        let mode = PresentationMode::new().with_fullscreen(FullscreenMode::Kiosk);

        assert_eq!(
            backend.set_presentation(id, &mode),
            WindowOperationResult::Unsupported
        );
        assert_eq!(
            backend.window(id).unwrap().properties().state,
            WindowState::Normal
        );
        assert!(backend.lock().pending_property_updates.is_empty());
    }

    /// §6.5: the optional controls are unwired, so asking for one downgrades an
    /// otherwise native borderless presentation to conditional.
    #[test]
    fn presentation_dropped_controls_downgrade_to_conditional() {
        let mut backend = backend();
        let id = backend
            .create_window(CreateWindow::new(WindowProperties::default()))
            .unwrap();
        let base = PresentationMode::new().with_fullscreen(FullscreenMode::Borderless);

        let asks = [
            base.hide_cursor(true),
            base.inhibit_sleep(true),
            base.on_display(DisplayId::from_raw(0)),
            base.with_safe_escape(SafeEscapePolicy::Disabled),
        ];
        for mode in asks {
            assert_eq!(
                backend.set_presentation(id, &mode),
                WindowOperationResult::conditional(),
                "a dropped control must not be reported native: {mode:?}"
            );
        }
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
        // Creation-time only: no post-creation toggle exists.
        assert_eq!(
            backend.supports(WindowFeature::Transparency),
            SupportLevel::Limited
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

    fn a_display(raw: u64, width: f32) -> Display {
        Display::new(
            DisplayId::from_raw(raw),
            obscur_windowing::display::DisplayMode::new(Size::new(width, 1080.0)),
        )
    }

    /// §6.6: before the first loop turn there is no topology to report, and the
    /// port must say so rather than invent one.
    #[test]
    fn displays_are_empty_until_the_handler_has_enumerated() {
        let mut backend = backend();
        assert!(backend.displays().is_empty());
        assert_eq!(backend.primary_display(), None);
        assert!(backend.poll_display_events().is_empty());
    }

    /// §6.6: `displays()` hands out the topology the handler enumerated, whole —
    /// the port is a reader of that snapshot, not a source of defaults.
    #[test]
    fn displays_report_the_enumerated_topology() {
        let backend = backend();
        let enumerated = vec![
            a_display(0, 1920.0).primary(true),
            a_display(1, 3840.0).with_name("DELL U2720Q"),
        ];
        backend.lock().record_displays(enumerated.clone());

        assert_eq!(backend.displays(), enumerated);
        assert_eq!(backend.primary_display(), Some(DisplayId::from_raw(0)));
    }

    /// A topology with no monitor marked primary — winit reports none on Wayland
    /// — must not promote the first one.
    #[test]
    fn primary_display_is_none_when_no_monitor_is_primary() {
        let backend = backend();
        backend
            .lock()
            .record_displays(vec![a_display(0, 1920.0), a_display(1, 3840.0)]);

        assert_eq!(backend.displays().len(), 2);
        assert_eq!(backend.primary_display(), None);
    }

    /// §6.6: the derived hotplug events reach the port's poll, and drain.
    #[test]
    fn poll_display_events_drains_what_enumeration_derived() {
        let mut backend = backend();
        backend
            .lock()
            .record_displays(vec![a_display(0, 1920.0).primary(true)]);

        assert_eq!(
            backend.poll_display_events(),
            vec![
                DisplayEvent::Connected(DisplayId::from_raw(0)),
                DisplayEvent::PrimaryChanged(DisplayId::from_raw(0)),
            ]
        );
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
        assert_eq!(
            backend.supports(WindowFeature::AlwaysOnTop),
            SupportLevel::Native
        );
    }
}
