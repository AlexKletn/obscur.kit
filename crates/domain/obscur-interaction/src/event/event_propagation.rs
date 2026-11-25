//! Mutable propagation control carried by an event as it routes (spec §4.7).

#[doc = include_str!("../../docs/api/event/event_propagation.doc/EventPropagation.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventPropagation {
    /// Propagation should stop after the current node finishes (`stopPropagation`).
    pub stopped: bool,
    /// Propagation should stop immediately, skipping any remaining handlers on
    /// the current node (`stopImmediatePropagation`).
    pub stop_immediate: bool,
    /// The host's default action has been suppressed (`preventDefault`).
    pub default_prevented: bool,
}

impl EventPropagation {
    /// Creates fresh propagation control with every flag clear.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            stopped: false,
            stop_immediate: false,
            default_prevented: false,
        }
    }

    /// Requests that propagation stop after the current node finishes
    /// (`stopPropagation`).
    pub const fn stop_propagation(&mut self) {
        self.stopped = true;
    }

    /// Requests that propagation stop immediately, skipping any remaining
    /// handlers on the current node (`stopImmediatePropagation`). This also
    /// implies [`stop_propagation`](Self::stop_propagation).
    pub const fn stop_immediate_propagation(&mut self) {
        self.stop_immediate = true;
        self.stopped = true;
    }

    /// Suppresses the host's default action for this event (`preventDefault`).
    pub const fn prevent_default(&mut self) {
        self.default_prevented = true;
    }

    /// Returns whether propagation has been stopped by either the ordinary or
    /// the immediate form.
    #[must_use]
    pub const fn is_stopped(&self) -> bool {
        self.stopped || self.stop_immediate
    }

    /// Returns whether the host's default action has been suppressed.
    #[must_use]
    pub const fn is_default_prevented(&self) -> bool {
        self.default_prevented
    }
}

impl Default for EventPropagation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_all_clear() {
        let p = EventPropagation::new();
        assert!(!p.is_stopped());
        assert!(!p.is_default_prevented());
        assert_eq!(p, EventPropagation::default());
    }

    #[test]
    fn stop_propagation_sets_stopped() {
        let mut p = EventPropagation::new();
        p.stop_propagation();
        assert!(p.stopped);
        assert!(!p.stop_immediate);
        assert!(p.is_stopped());
    }

    #[test]
    fn stop_immediate_also_sets_stopped() {
        let mut p = EventPropagation::new();
        p.stop_immediate_propagation();
        assert!(p.stop_immediate);
        assert!(p.stopped);
        assert!(p.is_stopped());
    }

    #[test]
    fn prevent_default_sets_flag() {
        let mut p = EventPropagation::new();
        p.prevent_default();
        assert!(p.is_default_prevented());
        assert!(!p.is_stopped());
    }
}
