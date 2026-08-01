#![doc = include_str!("event_propagation.doc/_module.md")]

#[doc = include_str!("event_propagation.doc/EventPropagation.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventPropagation {
    #[doc = include_str!("event_propagation.doc/stopped.md")]
    pub stopped: bool,
    #[doc = include_str!("event_propagation.doc/stop_immediate.md")]
    pub stop_immediate: bool,
    #[doc = include_str!("event_propagation.doc/default_prevented.md")]
    pub default_prevented: bool,
}

impl EventPropagation {
    #[doc = include_str!("event_propagation.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            stopped: false,
            stop_immediate: false,
            default_prevented: false,
        }
    }

    #[doc = include_str!("event_propagation.doc/stop_propagation.md")]
    pub const fn stop_propagation(&mut self) {
        self.stopped = true;
    }

    #[doc = include_str!("event_propagation.doc/stop_immediate_propagation.md")]
    pub const fn stop_immediate_propagation(&mut self) {
        self.stop_immediate = true;
        self.stopped = true;
    }

    #[doc = include_str!("event_propagation.doc/prevent_default.md")]
    pub const fn prevent_default(&mut self) {
        self.default_prevented = true;
    }

    #[doc = include_str!("event_propagation.doc/is_stopped.md")]
    #[must_use]
    pub const fn is_stopped(&self) -> bool {
        self.stopped || self.stop_immediate
    }

    #[doc = include_str!("event_propagation.doc/is_default_prevented.md")]
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
