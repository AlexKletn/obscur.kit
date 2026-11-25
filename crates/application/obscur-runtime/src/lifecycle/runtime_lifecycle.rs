//! RuntimeLifecycle (skeleton stub for the spec-defined entity; not yet implemented).

use std::collections::HashSet;

use obscur_identity::WindowId;

use super::{LifecycleEvent, ShutdownConcern, ShutdownRequest};

/// Tracks host-reported windows and queues notifications. The host decides how to respond.
#[derive(Debug, Default)]
pub struct RuntimeLifecycle {
    /// IDs of the windows currently open according to the host.
    windows: HashSet<WindowId>,
    /// Notifications waiting for the next host drain.
    events: Vec<LifecycleEvent>,
}

impl RuntimeLifecycle {
    /// Records a critical stop so the host can report the failure and release resources.
    pub fn report_critical_stop(&mut self, concern: ShutdownConcern) {
        self.events.push(LifecycleEvent::CriticalStop(concern));
    }

    /// Queues a shutdown request. The host chooses when and how to shut down.
    pub fn request_shutdown(&mut self, request: ShutdownRequest) {
        self.events.push(LifecycleEvent::ShutdownRequested(request));
    }

    /// Tracks an open window. Repeated reports for the same ID count once.
    pub fn window_opened(&mut self, window: WindowId) {
        self.windows.insert(window);
    }

    /// Reports a window closure. A transition from one tracked window to zero queues an event. Unknown and repeated closures have no effect.
    pub fn window_closed(&mut self, window: WindowId) {
        if self.windows.remove(&window) && self.windows.is_empty() {
            self.events.push(LifecycleEvent::LastWindowClosed(window));
        }
    }

    /// Drains pending notifications in order.
    #[must_use]
    pub fn take_events(&mut self) -> Vec<LifecycleEvent> {
        std::mem::take(&mut self.events)
    }
}
