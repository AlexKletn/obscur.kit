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
use obscur_windowing::display::{Display, DisplayEvent, DisplayId};
use obscur_windowing::request::CreateWindow;
use obscur_windowing::window::{WindowEvent, WindowProperties};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::WindowEvent as WinitWindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::monitor::MonitorHandle;
use winit::window::{WindowAttributes, WindowId as WinitWindowId, WindowLevel};

use crate::conversion::{DisplayConverter, WindowEventConverter};
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
    /// The display topology as of the last enumeration, which is what the
    /// backend's `displays()` hands out. Monitors, like windows, are only
    /// reachable through an [`ActiveEventLoop`], so the pull-based port reads
    /// this snapshot rather than the platform (spec §6.6).
    pub(crate) displays: Vec<Display>,
    /// Display hotplug/configuration events awaiting the next
    /// `poll_display_events` drain.
    pub(crate) display_events: Vec<DisplayEvent>,
    /// The [`DisplayId`] minted for each monitor identity, so an id stays
    /// attached to one monitor across re-enumerations.
    display_ids: HashMap<MonitorKey, DisplayId>,
    /// The source of fresh display identities.
    next_display_id: u64,
}

/// The identity by which a monitor is recognized across enumerations (spec
/// §6.6).
///
/// winit exposes no stable monitor identifier — only a name, which is the
/// connector on X11/Wayland and the adapter path on Windows (unique in both),
/// but which macOS repeats for two identical panels. The ordinal disambiguates
/// those by enumeration order.
///
/// Geometry is deliberately **not** part of the key: a monitor whose mode,
/// scale, or position changed is the same monitor and must keep its id, or
/// every reconfiguration would report as a disconnect followed by a connect.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MonitorKey {
    /// The monitor name winit reports, or `None` when it reports none.
    name: Option<String>,
    /// Position among the same-named monitors of one enumeration.
    ordinal: usize,
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

    /// Replaces the display topology with a fresh enumeration, reporting what
    /// changed as [`DisplayEvent`]s (spec §6.6 "hotplug/configuration events").
    ///
    /// winit 0.30 delivers no monitor notification of any kind, so the events
    /// §6.6 requires can only be *derived*, by diffing one enumeration against
    /// the last. That diff is the decision in this subsystem, and it lives here
    /// rather than in the handler for the same reason focus tracking does: the
    /// handler's callbacks only run inside a real winit loop.
    pub(crate) fn record_displays(&mut self, displays: Vec<Display>) {
        for previous in &self.displays {
            if !displays.iter().any(|display| display.id() == previous.id()) {
                self.display_events
                    .push(DisplayEvent::Disconnected(previous.id()));
            }
        }
        for display in &displays {
            match self
                .displays
                .iter()
                .find(|previous| previous.id() == display.id())
            {
                None => self
                    .display_events
                    .push(DisplayEvent::Connected(display.id())),
                Some(previous) if configuration_changed(previous, display) => self
                    .display_events
                    .push(DisplayEvent::ConfigurationChanged(display.id())),
                Some(_) => {}
            }
        }

        let was_primary = primary_of(&self.displays);
        let is_primary = primary_of(&displays);
        if is_primary != was_primary
            && let Some(id) = is_primary
        {
            self.display_events.push(DisplayEvent::PrimaryChanged(id));
        }

        self.displays = displays;
    }

    /// Returns the [`DisplayId`] for a monitor named `name` appearing at
    /// `ordinal` among its same-named peers, minting one on first sight.
    ///
    /// Ids are retained forever, not cleared when a monitor disappears: a
    /// monitor unplugged and plugged back in is the same monitor to the user,
    /// and a caller holding its id across the gap should find it again rather
    /// than meet a stranger.
    pub(crate) fn display_id_for(&mut self, name: Option<String>, ordinal: usize) -> DisplayId {
        let key = MonitorKey { name, ordinal };
        if let Some(&id) = self.display_ids.get(&key) {
            return id;
        }
        let id = DisplayId::from_raw(self.next_display_id);
        self.next_display_id += 1;
        self.display_ids.insert(key, id);
        id
    }
}

/// Returns the id of the primary display in `displays`, if one is marked.
fn primary_of(displays: &[Display]) -> Option<DisplayId> {
    displays
        .iter()
        .find(|display| display.is_primary())
        .map(Display::id)
}

/// Returns whether `current` reports a different configuration than `previous`
/// for the same display (spec §6.6).
///
/// The comparison is exactly what [`DisplayEvent::ConfigurationChanged`]
/// documents — mode, scale, or bounds — plus the work area, which is the
/// reserved-region half of bounds. Primacy is excluded because it has its own
/// [`DisplayEvent::PrimaryChanged`], and reporting a primary flip twice would
/// make a consumer that re-lays-out on a configuration change do so for a change
/// that moved no geometry. The name is excluded because it is the identity this
/// display was matched by in the first place.
fn configuration_changed(previous: &Display, current: &Display) -> bool {
    previous.current_mode() != current.current_mode()
        || previous.available_modes() != current.available_modes()
        || previous.scale_factor() != current.scale_factor()
        || previous.logical_bounds() != current.logical_bounds()
        || previous.physical_bounds() != current.physical_bounds()
        || previous.work_area() != current.work_area()
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

    /// Materializes deferred windows, applies queued operations, and refreshes
    /// the display topology (spec §6.1, §6.6).
    ///
    /// Runs on every loop turn that has an [`ActiveEventLoop`]: it re-enumerates
    /// the monitors, drains the pending-create queue (building each real winit
    /// window and emitting a [`WindowEvent::Created`]), pushes queued property
    /// sets onto live windows, grants queued focus, and drops closed windows.
    fn pump(&self, event_loop: &ActiveEventLoop) {
        let mut state = self.lock();
        pump_displays(event_loop, &mut state);

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

/// Re-enumerates the platform's monitors into `state` (spec §6.6).
///
/// This is the only place a [`MonitorHandle`](winit::monitor::MonitorHandle) is
/// reachable, so the pull-based `displays()` is served from what this leaves
/// behind. It runs on every loop turn rather than once at startup because winit
/// 0.30 has no monitor notification to run on instead — a topology refreshed
/// only at `resumed` would still describe an unplugged monitor. The re-read is
/// what it costs to not be stale, and the cost is platform-shaped: X11 and
/// Wayland answer from winit's own cached output list, while Windows
/// re-enumerates through `EnumDisplayMonitors` on every call.
/// [`BackendState::record_displays`] makes the write idempotent, so an unchanged
/// topology enqueues nothing.
fn pump_displays(event_loop: &ActiveEventLoop, state: &mut BackendState) {
    let monitors: Vec<_> = event_loop.available_monitors().collect();
    let converter = if global_coordinates_are_published(&monitors) {
        DisplayConverter::new()
    } else {
        DisplayConverter::without_global_coordinates()
    };
    // winit reports `None` here on Wayland and Web, where no primary display
    // exists to name; every display then stays non-primary, which is the truth.
    let primary = event_loop.primary_monitor();

    let mut ordinals: HashMap<Option<String>, usize> = HashMap::new();
    let mut displays = Vec::new();
    for monitor in monitors {
        let name = monitor.name();
        let ordinal = ordinals.entry(name.clone()).or_default();
        let id = state.display_id_for(name, *ordinal);
        *ordinal += 1;
        displays.push(
            converter
                .convert(id, &monitor)
                .primary(primary.as_ref() == Some(&monitor)),
        );
    }
    state.record_displays(displays);
}

/// Returns whether `monitors` carry where each one sits in a global coordinate
/// space (spec §6.6: "On Wayland, *unavailable* global coordinates are
/// represented honestly").
///
/// The spec's condition is unavailability, not a platform, and the two are not
/// the same: a Wayland compositor publishes each output's logical position
/// through **xdg-output**, and winit reports it (`wayland/output.rs`'s
/// `position()` reads `logical_position`, falling back to `wl_output.geometry`'s
/// `location` — neither is hardcoded). Mutter, KWin and wlroots all implement it.
/// Withholding is a *may* in the protocol, not a rule, so keying off `is_wayland()`
/// blanks a topology the compositor published in full — which is the §6.6 rule
/// inverted on the one platform §6.6 names.
///
/// What is genuinely undetectable is a compositor that withholds by reporting
/// `(0, 0)` for everything: winit cannot distinguish that from a real origin. But
/// that ambiguity only exists when **every** monitor claims the origin, which no
/// real multi-monitor topology does — two monitors cannot both start at `(0, 0)`.
/// A single monitor at the origin is simply where it is. So the absence is
/// detected from the positions themselves and needs no `cfg`.
fn global_coordinates_are_published(monitors: &[MonitorHandle]) -> bool {
    let positions: Vec<_> = monitors
        .iter()
        .map(|monitor| {
            let position = monitor.position();
            (position.x, position.y)
        })
        .collect();
    positions_carry_a_topology(&positions)
}

/// The decision behind [`global_coordinates_are_published`], over plain positions
/// so it is reachable without an event loop (a `MonitorHandle` needs one, which is
/// why the rule this replaced was never tested).
fn positions_carry_a_topology(positions: &[(i32, i32)]) -> bool {
    positions.len() < 2 || !positions.iter().all(|&position| position == (0, 0))
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
    use obscur_geometry::{Point, Rect, ScaleFactor, Size};
    use obscur_windowing::window::WindowConstraints;
    use winit::dpi::Size as WinitSize;

    /// §6.6's condition is *unavailable* coordinates, not a platform. A second
    /// monitor sitting at a real offset is a published topology on Wayland exactly
    /// as on X11 — winit reads it from xdg-output — so it must not be blanked.
    #[test]
    fn a_monitor_at_a_real_offset_carries_a_topology() {
        assert!(positions_carry_a_topology(&[(0, 0), (1920, 120)]));
    }

    /// The one case winit genuinely cannot read: a compositor that withholds by
    /// answering the origin for everything. Two monitors cannot both start at
    /// `(0, 0)`, so the topology is absent rather than degenerate.
    #[test]
    fn every_monitor_at_the_origin_is_a_withheld_topology() {
        assert!(!positions_carry_a_topology(&[(0, 0), (0, 0)]));
        assert!(!positions_carry_a_topology(&[(0, 0), (0, 0), (0, 0)]));
    }

    /// A lone monitor at the origin is simply where it is — there is nothing for a
    /// compositor to withhold, so blanking it would invent an absence.
    #[test]
    fn a_single_monitor_at_the_origin_is_where_it_is() {
        assert!(positions_carry_a_topology(&[(0, 0)]));
        assert!(positions_carry_a_topology(&[]));
    }

    /// Negative coordinates are ordinary: a monitor placed left of or above the
    /// primary is a published position, not an absence.
    #[test]
    fn a_monitor_left_of_the_primary_carries_a_topology() {
        assert!(positions_carry_a_topology(&[(-1920, 0), (0, 0)]));
    }

    /// §6.2 min/max constraints are creation-time attributes in winit, so a
    /// request carrying them must reach the platform rather than sit unread on
    /// the `CreateWindow`.
    #[test]
    fn constraints_reach_the_winit_attributes() {
        let request = CreateWindow::new(WindowProperties::default()).with_constraints(
            WindowConstraints::new(
                Some(Size::new(320.0, 240.0)),
                Some(Size::new(1280.0, 800.0)),
            ),
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

    fn a_display(raw: u64, width: f32) -> Display {
        Display::new(
            DisplayId::from_raw(raw),
            obscur_windowing::display::DisplayMode::new(Size::new(width, 1080.0)),
        )
    }

    /// §6.6: an id must survive re-enumeration, or a caller holding one would
    /// find it pointing at a different monitor after any loop turn.
    #[test]
    fn display_ids_are_stable_across_enumerations() {
        let mut state = BackendState::default();
        let first = state.display_id_for(Some("eDP-1".to_owned()), 0);
        let second = state.display_id_for(Some("HDMI-1".to_owned()), 0);
        assert_ne!(first, second);

        // The same monitors, enumerated again: the same ids.
        assert_eq!(state.display_id_for(Some("eDP-1".to_owned()), 0), first);
        assert_eq!(state.display_id_for(Some("HDMI-1".to_owned()), 0), second);
        // Even after one of them went away and came back.
        assert_eq!(state.display_id_for(Some("eDP-1".to_owned()), 0), first);
    }

    /// Two monitors winit reports under one name — macOS does this for identical
    /// panels — are two displays, not one.
    #[test]
    fn same_named_monitors_are_told_apart_by_ordinal() {
        let mut state = BackendState::default();
        let name = Some("LG HDR 4K".to_owned());
        let first = state.display_id_for(name.clone(), 0);
        let second = state.display_id_for(name.clone(), 1);
        assert_ne!(first, second);
        assert_eq!(state.display_id_for(name, 1), second);

        // Nameless monitors are distinguished the same way.
        assert_ne!(state.display_id_for(None, 0), state.display_id_for(None, 1));
    }

    /// §6.6: the first enumeration is a topology arriving, so a consumer that
    /// only reads events learns the whole of it.
    #[test]
    fn the_first_enumeration_connects_every_display() {
        let mut state = BackendState::default();
        state.record_displays(vec![
            a_display(0, 1920.0).primary(true),
            a_display(1, 3840.0),
        ]);

        assert_eq!(
            state.display_events,
            vec![
                DisplayEvent::Connected(DisplayId::from_raw(0)),
                DisplayEvent::Connected(DisplayId::from_raw(1)),
                DisplayEvent::PrimaryChanged(DisplayId::from_raw(0)),
            ]
        );
    }

    /// The enumeration runs every loop turn, so an unchanged topology must
    /// derive nothing at all — otherwise `poll_display_events` would report a
    /// hotplug per frame.
    #[test]
    fn re_enumerating_an_unchanged_topology_reports_nothing() {
        let mut state = BackendState::default();
        let topology = vec![a_display(0, 1920.0).primary(true), a_display(1, 3840.0)];
        state.record_displays(topology.clone());
        state.display_events.clear();

        state.record_displays(topology.clone());
        assert!(state.display_events.is_empty());
        assert_eq!(state.displays, topology);
    }

    /// §6.6 hotplug: a monitor appearing and one vanishing each report once, and
    /// the untouched monitor reports nothing.
    #[test]
    fn connecting_and_disconnecting_report_only_the_monitors_that_moved() {
        let mut state = BackendState::default();
        state.record_displays(vec![a_display(0, 1920.0), a_display(1, 3840.0)]);
        state.display_events.clear();

        // Display 1 unplugged, display 2 plugged in; display 0 untouched.
        state.record_displays(vec![a_display(0, 1920.0), a_display(2, 2560.0)]);

        assert_eq!(
            state.display_events,
            vec![
                DisplayEvent::Disconnected(DisplayId::from_raw(1)),
                DisplayEvent::Connected(DisplayId::from_raw(2)),
            ]
        );
    }

    /// §6.6 configuration: a monitor that changed mode is the *same* monitor, so
    /// it must report a configuration change and keep its id — never a
    /// disconnect followed by a connect.
    #[test]
    fn a_reconfigured_display_keeps_its_id_and_reports_a_change() {
        let mut state = BackendState::default();
        state.record_displays(vec![a_display(0, 1920.0)]);
        state.display_events.clear();

        state.record_displays(vec![a_display(0, 2560.0)]);
        assert_eq!(
            state.display_events,
            vec![DisplayEvent::ConfigurationChanged(DisplayId::from_raw(0))]
        );

        // Scale and bounds are configuration too, each on its own.
        state.display_events.clear();
        state.record_displays(vec![
            a_display(0, 2560.0).with_scale_factor(ScaleFactor::new(2.0)),
        ]);
        assert_eq!(
            state.display_events,
            vec![DisplayEvent::ConfigurationChanged(DisplayId::from_raw(0))]
        );

        state.display_events.clear();
        state.record_displays(vec![
            a_display(0, 2560.0)
                .with_scale_factor(ScaleFactor::new(2.0))
                .with_logical_bounds(Rect::new(Point::new(0.0, 0.0), Size::new(1280.0, 540.0))),
        ]);
        assert_eq!(
            state.display_events,
            vec![DisplayEvent::ConfigurationChanged(DisplayId::from_raw(0))]
        );
    }

    /// The primary moving is a `PrimaryChanged`, and *only* that: no geometry
    /// moved, so a consumer that re-lays-out on a configuration change must not
    /// be woken.
    #[test]
    fn the_primary_moving_reports_primary_changed_alone() {
        let mut state = BackendState::default();
        state.record_displays(vec![
            a_display(0, 1920.0).primary(true),
            a_display(1, 3840.0),
        ]);
        state.display_events.clear();

        state.record_displays(vec![
            a_display(0, 1920.0),
            a_display(1, 3840.0).primary(true),
        ]);
        assert_eq!(
            state.display_events,
            vec![DisplayEvent::PrimaryChanged(DisplayId::from_raw(1))]
        );
    }

    /// Losing the primary with nothing to take its place — every monitor
    /// unplugged but one on Wayland, where winit names no primary — reports the
    /// disconnect and stays quiet about primacy rather than crowning a survivor.
    #[test]
    fn losing_the_primary_crowns_no_replacement() {
        let mut state = BackendState::default();
        state.record_displays(vec![
            a_display(0, 1920.0).primary(true),
            a_display(1, 3840.0),
        ]);
        state.display_events.clear();

        state.record_displays(vec![a_display(1, 3840.0)]);
        assert_eq!(
            state.display_events,
            vec![DisplayEvent::Disconnected(DisplayId::from_raw(0))]
        );
        assert!(!state.displays[0].is_primary());
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
