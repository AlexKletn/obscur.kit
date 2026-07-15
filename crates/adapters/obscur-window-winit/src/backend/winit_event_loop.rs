//! The winit [`ApplicationHandler`] integration that bridges winit's
//! callback-driven event loop to the pull-based [`WindowBackend`] port
//! (spec §6.1 windowing; §2.7 "Реализует `WindowBackend` и преобразует native
//! events в доменные события"; §3.5 frame scheduling).
//!
//! This module owns the three tightly-coupled pieces of that bridge: the
//! [`UserEvent`] the frame scheduler sends to wake the loop, the shared
//! [`BackendState`] queue both the backend handle and the handler mutate, and
//! the [`WinitAppHandler`] that drains it inside winit callbacks.

use std::collections::HashMap;
use std::mem;
use std::sync::{Arc, Mutex};

use obscur_identity::WindowId;
use obscur_windowing::request::CreateWindow;
use obscur_windowing::window::{WindowEvent, WindowProperties};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::WindowEvent as WinitWindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{WindowAttributes, WindowId as WinitWindowId, WindowLevel};

use crate::conversion::WindowEventConverter;
use crate::error::WinitBackendError;

use super::winit_window::WinitWindow;

/// A wake-up message sent to the winit event loop from any thread (spec §3.5).
///
/// The §3.5 frame scheduler runs off the event loop; when it decides a frame is
/// due it sends a `UserEvent` through an
/// [`EventLoopProxy`](winit::event_loop::EventLoopProxy) (wrapped by
/// [`EventLoopWaker`](crate::wake::EventLoopWaker)) to nudge the loop awake.
/// There is a single variant today, but the enum keeps the door open for
/// additional cross-thread signals without changing the event-loop type
/// parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserEvent {
    /// Wake the event loop so it can service a scheduled frame.
    WakeUp,
}

/// The winit-side mutable state shared between [`WinitWindowBackend`] and the
/// [`WinitAppHandler`] (spec §6.1).
///
/// This is the shared-queue half of the pull/callback bridge. The backend
/// handle enqueues work here — window creation, property reconciliation, focus,
/// and close requests — and the handler drains that work inside winit callbacks
/// (where an [`ActiveEventLoop`] and the live windows are reachable), pushing
/// converted [`WindowEvent`]s back for the backend to poll. It is deliberately
/// *not* where the domain [`Window`](obscur_windowing::window::Window) records
/// live: those are owned directly by the backend so `window()` can hand out a
/// `&Window` without borrowing through a mutex guard.
///
/// [`WinitWindowBackend`]: super::WinitWindowBackend
#[derive(Debug, Default)]
pub(crate) struct BackendState {
    /// Windows requested via `create_window` but not yet materialized because no
    /// [`ActiveEventLoop`] has been available to build them.
    pub(crate) pending_creates: Vec<(WindowId, CreateWindow)>,
    /// Windows requested to close; the handler drops the live winit window.
    pub(crate) pending_closes: Vec<WindowId>,
    /// Desired full property sets to push onto live windows (from updates,
    /// chrome-mode changes, and presentation changes).
    pub(crate) pending_property_updates: Vec<(WindowId, WindowProperties)>,
    /// Windows requested to receive keyboard focus.
    pub(crate) pending_focus: Vec<WindowId>,
    /// Live winit windows keyed by the domain id that minted them.
    pub(crate) windows: HashMap<WindowId, WinitWindow>,
    /// Reverse map from winit's own window id to the domain id, for routing
    /// native events back to the right window.
    pub(crate) winit_to_domain: HashMap<WinitWindowId, WindowId>,
    /// Converted window events awaiting the next `poll_window_events` drain.
    pub(crate) window_events: Vec<(WindowId, WindowEvent)>,
    /// The window the compositor last told us holds keyboard focus, tracked from
    /// the native `Focused` events. This is the only *observed* activation state
    /// the backend has: winit grants focus asynchronously, so `activate` reads
    /// this rather than assuming its request was honored (spec §6.3).
    pub(crate) focused: Option<WindowId>,
    /// Window-creation failures recorded by the handler for later inspection;
    /// creation is deferred, so its errors cannot be returned from
    /// `create_window` and are surfaced here instead.
    pub(crate) creation_errors: Vec<WinitBackendError>,
}

impl BackendState {
    /// Buffers a converted `event` for `id`, keeping [`BackendState::focused`] in
    /// step with the compositor's focus verdict (spec §6.3).
    ///
    /// Focus tracking lives here rather than inline in the handler so it is
    /// reachable without an [`ActiveEventLoop`]: the handler's callback can only
    /// run inside a real winit loop, but this — the part with a decision in it —
    /// is what `activate` depends on.
    pub(crate) fn record_window_event(&mut self, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::FocusGained => self.focused = Some(id),
            WindowEvent::FocusLost if self.focused == Some(id) => self.focused = None,
            _ => {}
        }
        self.window_events.push((id, event));
    }
}

/// The winit [`ApplicationHandler`] that services the shared [`BackendState`]
/// inside the event loop (spec §6.1, §2.7).
///
/// A host builds one via [`WinitWindowBackend::app_handler`] so it shares the
/// exact same [`BackendState`] as the backend handle, then hands it to
/// [`run_app`](winit::event_loop::EventLoop::run_app). On each turn the handler
/// materializes any deferred windows, applies queued property/focus/close
/// operations, and converts native window events into domain [`WindowEvent`]s.
///
/// [`WinitWindowBackend::app_handler`]: super::WinitWindowBackend::app_handler
#[derive(Debug)]
pub struct WinitAppHandler {
    /// The shared winit-side state, cloned from the backend handle.
    state: Arc<Mutex<BackendState>>,
    /// The pure native→domain window-event converter.
    window_events: WindowEventConverter,
}

impl WinitAppHandler {
    /// Builds a handler sharing `state` with its originating backend handle.
    pub(crate) fn new(state: Arc<Mutex<BackendState>>) -> Self {
        Self {
            state,
            window_events: WindowEventConverter::new(),
        }
    }

    /// Materializes deferred windows and applies queued operations (spec §6.1).
    ///
    /// Runs on every loop turn that has an [`ActiveEventLoop`]: it drains the
    /// pending-create queue (building each real winit window and emitting a
    /// [`WindowEvent::Created`]), pushes queued property sets onto live windows,
    /// grants queued focus, and drops closed windows.
    fn pump(&self, event_loop: &ActiveEventLoop) {
        let mut state = self.lock();

        for (id, request) in mem::take(&mut state.pending_creates) {
            let attributes = window_attributes(&request);
            match event_loop.create_window(attributes) {
                Ok(window) => {
                    let winit_id = window.id();
                    let winit_window = WinitWindow::new(id, Arc::new(window));
                    state.winit_to_domain.insert(winit_id, id);
                    state.windows.insert(id, winit_window);
                    state.window_events.push((id, WindowEvent::Created));
                }
                Err(error) => {
                    state
                        .creation_errors
                        .push(WinitBackendError::WindowCreation(error.to_string().into()));
                }
            }
        }

        for (id, properties) in mem::take(&mut state.pending_property_updates) {
            if let Some(window) = state.windows.get(&id) {
                window.apply_properties(&properties);
            }
        }

        for id in mem::take(&mut state.pending_focus) {
            if let Some(window) = state.windows.get(&id) {
                window.winit().focus_window();
            }
        }

        for id in mem::take(&mut state.pending_closes) {
            if let Some(window) = state.windows.remove(&id) {
                state.winit_to_domain.remove(&window.winit().id());
            }
            if state.focused == Some(id) {
                state.focused = None;
            }
        }
    }

    /// Locks the shared state, treating a poisoned mutex as an unrecoverable bug.
    fn lock(&self) -> std::sync::MutexGuard<'_, BackendState> {
        self.state.lock().expect("backend state mutex poisoned")
    }
}

impl ApplicationHandler<UserEvent> for WinitAppHandler {
    /// Materializes deferred windows once the platform is ready to create them.
    ///
    /// Some platforms (notably Android/Web) forbid window creation before the
    /// first `resumed`; deferring creation until here — and re-pumping on every
    /// `about_to_wait` — is what lets `create_window` return an id synchronously
    /// yet still produce a real window.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.pump(event_loop);
    }

    /// Routes a native window event to its domain window, converting it.
    ///
    /// Only lifecycle/geometry/focus/scale/occlusion events convert to a domain
    /// [`WindowEvent`]; keyboard and pointer input are the interaction crate's
    /// concern and out of this port's `poll_window_events`, so they are not
    /// buffered here (the [`KeyboardConverter`](crate::conversion::KeyboardConverter)
    /// / [`PointerConverter`](crate::conversion::PointerConverter) feed a separate
    /// input path in a later sub-step). An event for an unknown winit window is
    /// dropped.
    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        winit_window_id: WinitWindowId,
        event: WinitWindowEvent,
    ) {
        let mut state = self.lock();
        let Some(&domain_id) = state.winit_to_domain.get(&winit_window_id) else {
            return;
        };
        if let Some(converted) = self.window_events.convert(&event) {
            state.record_window_event(domain_id, converted);
        }
    }

    /// Services queued backend operations before the loop blocks for input.
    ///
    /// Property/focus/close requests (and any late `create_window` calls) are
    /// enqueued by the backend between loop turns; re-pumping here applies them
    /// promptly without waiting for the next OS event.
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.pump(event_loop);
    }

    /// Handles a cross-thread wake-up by requesting a redraw of every live
    /// window (spec §3.5).
    ///
    /// The frame scheduler sends [`UserEvent::WakeUp`] purely to break the loop
    /// out of its wait; translating it into redraw requests is the honest way to
    /// turn "a frame is due" into winit's redraw-driven rendering model.
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::WakeUp => {
                let state = self.lock();
                for window in state.windows.values() {
                    window.winit().request_redraw();
                }
            }
        }
    }
}

/// Builds winit [`WindowAttributes`] from a domain [`CreateWindow`] request
/// (spec §6.1, §6.2).
///
/// The whole request is mapped, not just its [`WindowProperties`]: the §6.2
/// min/max [`WindowConstraints`](obscur_windowing::window::WindowConstraints)
/// are creation-time attributes in winit too, so they are pushed here rather
/// than dropped. Mapped: title, inner size, min/max inner size, resizability,
/// decorations, transparency, always-on-top level, and an explicit position when
/// one is requested. Post-creation-only concerns (state toggles) are applied
/// afterwards via [`WinitWindow::apply_properties`].
///
/// The request's [`WindowRelation`](obscur_windowing::window::WindowRelation) is
/// deliberately **not** mapped. winit's only ownership primitive is
/// `WindowAttributes::with_parent_window`, which builds a *child* window clipped
/// into the parent's client area (`WS_CHILD` on Windows, a child on X11) — not
/// the floating owned dialog §6.1 means — and winit has no modality primitive at
/// all. Rather than materialize a window that lies about its relation,
/// [`WinitWindowBackend`]'s `create_window` rejects any request carrying an owner
/// or a modality before it ever reaches this function.
///
/// [`WinitWindowBackend`]: super::WinitWindowBackend
fn window_attributes(request: &CreateWindow) -> WindowAttributes {
    let properties = request.properties();
    let mut attributes = WindowAttributes::default()
        .with_title(properties.title.to_string())
        .with_inner_size(LogicalSize::new(
            f64::from(properties.size.width),
            f64::from(properties.size.height),
        ))
        .with_resizable(properties.resizable)
        .with_decorations(properties.decorated)
        .with_transparent(properties.transparent)
        .with_window_level(if properties.always_on_top {
            WindowLevel::AlwaysOnTop
        } else {
            WindowLevel::Normal
        });
    if let Some(position) = properties.position {
        attributes = attributes.with_position(LogicalPosition::new(
            f64::from(position.x),
            f64::from(position.y),
        ));
    }
    let constraints = request.constraints();
    if let Some(min) = constraints.min_size {
        attributes = attributes.with_min_inner_size(LogicalSize::new(
            f64::from(min.width),
            f64::from(min.height),
        ));
    }
    if let Some(max) = constraints.max_size {
        attributes = attributes.with_max_inner_size(LogicalSize::new(
            f64::from(max.width),
            f64::from(max.height),
        ));
    }
    attributes
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Size;
    use obscur_windowing::window::WindowConstraints;
    use winit::dpi::Size as WinitSize;

    /// §6.2 min/max constraints are creation-time attributes in winit, so a
    /// request carrying them must reach the platform rather than sit unread on
    /// the `CreateWindow`.
    #[test]
    fn constraints_reach_the_winit_attributes() {
        let request = CreateWindow::new(WindowProperties::default()).with_constraints(
            WindowConstraints::new(Some(Size::new(320.0, 240.0)), Some(Size::new(1280.0, 800.0))),
        );

        let attributes = window_attributes(&request);

        assert_eq!(
            attributes.min_inner_size,
            Some(WinitSize::Logical(LogicalSize::new(320.0, 240.0)))
        );
        assert_eq!(
            attributes.max_inner_size,
            Some(WinitSize::Logical(LogicalSize::new(1280.0, 800.0)))
        );
    }

    /// An unconstrained request must leave both bounds unset rather than pin the
    /// window to a fabricated default.
    #[test]
    fn unconstrained_requests_leave_both_bounds_unset() {
        let attributes = window_attributes(&CreateWindow::new(WindowProperties::default()));
        assert_eq!(attributes.min_inner_size, None);
        assert_eq!(attributes.max_inner_size, None);
    }

    /// §6.3: the compositor's focus verdict is what `activate` reads back, so a
    /// native `Focused` event must land in `focused` and not only in the event
    /// buffer.
    #[test]
    fn focus_events_track_the_focused_window() {
        let mut state = BackendState::default();
        let first = WindowId::from_raw(1);
        let second = WindowId::from_raw(2);
        assert_eq!(state.focused, None);

        state.record_window_event(first, WindowEvent::FocusGained);
        assert_eq!(state.focused, Some(first));

        // Focus moving on is observed as the new window gaining it.
        state.record_window_event(second, WindowEvent::FocusGained);
        assert_eq!(state.focused, Some(second));

        // A stale loss from the window that already lost focus changes nothing.
        state.record_window_event(first, WindowEvent::FocusLost);
        assert_eq!(state.focused, Some(second));

        state.record_window_event(second, WindowEvent::FocusLost);
        assert_eq!(state.focused, None);

        // Every event still reaches the poll buffer regardless.
        assert_eq!(state.window_events.len(), 4);
    }

    /// Unrelated events must not disturb the focus record.
    #[test]
    fn non_focus_events_leave_focus_alone() {
        let mut state = BackendState::default();
        let id = WindowId::from_raw(1);
        state.record_window_event(id, WindowEvent::FocusGained);

        state.record_window_event(id, WindowEvent::Created);
        state.record_window_event(id, WindowEvent::Occluded(true));
        assert_eq!(state.focused, Some(id));
    }

    /// Either bound alone is honored on its own.
    #[test]
    fn a_single_bound_is_mapped_alone() {
        let min_only = window_attributes(
            &CreateWindow::new(WindowProperties::default())
                .with_constraints(WindowConstraints::with_min(Size::new(100.0, 50.0))),
        );
        assert_eq!(
            min_only.min_inner_size,
            Some(WinitSize::Logical(LogicalSize::new(100.0, 50.0)))
        );
        assert_eq!(min_only.max_inner_size, None);

        let max_only = window_attributes(
            &CreateWindow::new(WindowProperties::default())
                .with_constraints(WindowConstraints::with_max(Size::new(640.0, 480.0))),
        );
        assert_eq!(max_only.min_inner_size, None);
        assert_eq!(
            max_only.max_inner_size,
            Some(WinitSize::Logical(LogicalSize::new(640.0, 480.0)))
        );
    }
}
