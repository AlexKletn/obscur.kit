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
    /// Window-creation failures recorded by the handler for later inspection;
    /// creation is deferred, so its errors cannot be returned from
    /// `create_window` and are surfaced here instead.
    pub(crate) creation_errors: Vec<WinitBackendError>,
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
            let attributes = window_attributes(request.properties());
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
            state.window_events.push((domain_id, converted));
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

/// Builds winit [`WindowAttributes`] from the desired domain [`WindowProperties`]
/// (spec §6.1, §6.2).
///
/// Only the creation-time attributes winit accepts are mapped: title, inner
/// size, resizability, decorations, transparency, always-on-top level, and an
/// explicit position when one is requested. Post-creation-only concerns (state
/// toggles) are applied afterwards via [`WinitWindow::apply_properties`].
fn window_attributes(properties: &WindowProperties) -> WindowAttributes {
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
    attributes
}
