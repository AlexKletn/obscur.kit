use std::collections::HashMap;

use obscur_identity::WindowId;
use obscur_windowing::display::{Display, DisplayEvent, DisplayId};
use obscur_windowing::request::CreateWindow;
use obscur_windowing::window::{WindowEvent, WindowProperties};
use winit::window::WindowId as WinitWindowId;

use super::monitor_key::MonitorKey;
use super::winit_window::WinitWindow;
use crate::error::WinitBackendError;

#[doc = include_str!("../../docs/api/backend/winit_event_loop.doc/BackendState.md")]
#[derive(Debug, Default)]
pub(crate) struct BackendState {
    /// Windows requested via `create_window` but not yet materialized because no
    /// [`ActiveEventLoop`](winit::event_loop::ActiveEventLoop) has been available to build them.
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
    /// reachable through an [`ActiveEventLoop`](winit::event_loop::ActiveEventLoop), so the pull-based port reads
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

impl BackendState {
    #[doc = include_str!("../../docs/api/backend/winit_event_loop.doc/record_window_event.md")]
    pub(crate) fn record_window_event(&mut self, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::FocusGained => self.focused = Some(id),
            WindowEvent::FocusLost if self.focused == Some(id) => self.focused = None,
            _ => {}
        }
        self.window_events.push((id, event));
    }

    #[doc = include_str!("../../docs/api/backend/winit_event_loop.doc/record_displays.md")]
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

    #[doc = include_str!("../../docs/api/backend/winit_event_loop.doc/display_id_for.md")]
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

#[doc = include_str!("../../docs/api/backend/winit_event_loop.doc/configuration_changed.md")]
fn configuration_changed(previous: &Display, current: &Display) -> bool {
    previous.current_mode() != current.current_mode()
        || previous.available_modes() != current.available_modes()
        || previous.scale_factor() != current.scale_factor()
        || previous.logical_bounds() != current.logical_bounds()
        || previous.physical_bounds() != current.physical_bounds()
        || previous.work_area() != current.work_area()
}
