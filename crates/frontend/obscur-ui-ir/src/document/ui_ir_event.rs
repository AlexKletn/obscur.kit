//! `UiIrEvent`: an event binding wiring an event name to a handler (spec §2.5).

#[doc = include_str!("../../docs/api/document/ui_ir_event.doc/UiIrEvent.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiIrEvent {
    event: Box<str>,
    handler: Box<str>,
}

impl UiIrEvent {
    /// Creates an event binding wiring `event` to `handler`.
    #[must_use]
    pub fn new(event: impl Into<Box<str>>, handler: impl Into<Box<str>>) -> Self {
        Self {
            event: event.into(),
            handler: handler.into(),
        }
    }

    /// Returns the triggering event name (e.g. `"click"`).
    #[must_use]
    pub fn event(&self) -> &str {
        &self.event
    }

    /// Returns the command or action name that flows up when the event fires.
    #[must_use]
    pub fn handler(&self) -> &str {
        &self.handler
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_return_wired_names() {
        let event = UiIrEvent::new("click", "submit");
        assert_eq!(event.event(), "click");
        assert_eq!(event.handler(), "submit");
    }

    #[test]
    fn equal_bindings_compare_equal() {
        assert_eq!(
            UiIrEvent::new("click", "submit"),
            UiIrEvent::new("click", "submit")
        );
        assert_ne!(
            UiIrEvent::new("click", "submit"),
            UiIrEvent::new("click", "cancel")
        );
    }
}
