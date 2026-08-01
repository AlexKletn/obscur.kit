#![doc = include_str!("winit_event_loop.doc/_module.md")]

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

#[doc = include_str!("winit_event_loop.doc/UserEvent.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserEvent {
    #[doc = include_str!("winit_event_loop.doc/WakeUp.md")]
    WakeUp,
}

#[doc = include_str!("winit_event_loop.doc/BackendState.md")]
#[derive(Debug, Default)]
pub(crate) struct BackendState {
    #[doc = include_str!("winit_event_loop.doc/pending_creates.md")]
    pub(crate) pending_creates: Vec<(WindowId, CreateWindow)>,
    #[doc = include_str!("winit_event_loop.doc/pending_closes.md")]
    pub(crate) pending_closes: Vec<WindowId>,
    #[doc = include_str!("winit_event_loop.doc/pending_property_updates.md")]
    pub(crate) pending_property_updates: Vec<(WindowId, WindowProperties)>,
    #[doc = include_str!("winit_event_loop.doc/pending_focus.md")]
    pub(crate) pending_focus: Vec<WindowId>,
    #[doc = include_str!("winit_event_loop.doc/windows.md")]
    pub(crate) windows: HashMap<WindowId, WinitWindow>,
    #[doc = include_str!("winit_event_loop.doc/winit_to_domain.md")]
    pub(crate) winit_to_domain: HashMap<WinitWindowId, WindowId>,
    #[doc = include_str!("winit_event_loop.doc/window_events.md")]
    pub(crate) window_events: Vec<(WindowId, WindowEvent)>,
    #[doc = include_str!("winit_event_loop.doc/focused.md")]
    pub(crate) focused: Option<WindowId>,
    #[doc = include_str!("winit_event_loop.doc/creation_errors.md")]
    pub(crate) creation_errors: Vec<WinitBackendError>,
    #[doc = include_str!("winit_event_loop.doc/displays.md")]
    pub(crate) displays: Vec<Display>,
    #[doc = include_str!("winit_event_loop.doc/display_events.md")]
    pub(crate) display_events: Vec<DisplayEvent>,
    #[doc = include_str!("winit_event_loop.doc/display_ids.md")]
    display_ids: HashMap<MonitorKey, DisplayId>,
    #[doc = include_str!("winit_event_loop.doc/next_display_id.md")]
    next_display_id: u64,
}

#[doc = include_str!("winit_event_loop.doc/MonitorKey.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MonitorKey {
    #[doc = include_str!("winit_event_loop.doc/name.md")]
    name: Option<String>,
    #[doc = include_str!("winit_event_loop.doc/ordinal.md")]
    ordinal: usize,
}

impl BackendState {
    #[doc = include_str!("winit_event_loop.doc/record_window_event.md")]
    pub(crate) fn record_window_event(&mut self, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::FocusGained => self.focused = Some(id),
            WindowEvent::FocusLost if self.focused == Some(id) => self.focused = None,
            _ => {}
        }
        self.window_events.push((id, event));
    }

    #[doc = include_str!("winit_event_loop.doc/record_displays.md")]
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

    #[doc = include_str!("winit_event_loop.doc/display_id_for.md")]
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

#[doc = include_str!("winit_event_loop.doc/primary_of.md")]
fn primary_of(displays: &[Display]) -> Option<DisplayId> {
    displays
        .iter()
        .find(|display| display.is_primary())
        .map(Display::id)
}

#[doc = include_str!("winit_event_loop.doc/configuration_changed.md")]
fn configuration_changed(previous: &Display, current: &Display) -> bool {
    previous.current_mode() != current.current_mode()
        || previous.available_modes() != current.available_modes()
        || previous.scale_factor() != current.scale_factor()
        || previous.logical_bounds() != current.logical_bounds()
        || previous.physical_bounds() != current.physical_bounds()
        || previous.work_area() != current.work_area()
}

#[doc = include_str!("winit_event_loop.doc/WinitAppHandler.md")]
#[derive(Debug)]
pub struct WinitAppHandler {
    #[doc = include_str!("winit_event_loop.doc/state.md")]
    state: Arc<Mutex<BackendState>>,
    #[doc = include_str!("winit_event_loop.doc/window_events_2.md")]
    window_events: WindowEventConverter,
}

impl WinitAppHandler {
    #[doc = include_str!("winit_event_loop.doc/new.md")]
    pub(crate) fn new(state: Arc<Mutex<BackendState>>) -> Self {
        Self {
            state,
            window_events: WindowEventConverter::new(),
        }
    }

    #[doc = include_str!("winit_event_loop.doc/pump.md")]
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

    #[doc = include_str!("winit_event_loop.doc/lock.md")]
    fn lock(&self) -> std::sync::MutexGuard<'_, BackendState> {
        self.state.lock().expect("backend state mutex poisoned")
    }
}

impl ApplicationHandler<UserEvent> for WinitAppHandler {
    #[doc = include_str!("winit_event_loop.doc/resumed.md")]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.pump(event_loop);
    }

    #[doc = include_str!("winit_event_loop.doc/window_event.md")]
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

    #[doc = include_str!("winit_event_loop.doc/about_to_wait.md")]
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.pump(event_loop);
    }

    #[doc = include_str!("winit_event_loop.doc/user_event.md")]
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

#[doc = include_str!("winit_event_loop.doc/pump_displays.md")]
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

#[doc = include_str!("winit_event_loop.doc/global_coordinates_are_published.md")]
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

#[doc = include_str!("winit_event_loop.doc/positions_carry_a_topology.md")]
fn positions_carry_a_topology(positions: &[(i32, i32)]) -> bool {
    positions.len() < 2 || !positions.iter().all(|&position| position == (0, 0))
}

#[doc = include_str!("winit_event_loop.doc/window_attributes.md")]
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
