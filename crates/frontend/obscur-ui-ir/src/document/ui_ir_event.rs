//! `UiIrEvent`: an event binding wiring an event name to a handler (spec §2.5).

/// An event binding on an IR node: an event name wired to the handler or
/// command that fires *up* when it occurs (spec §2.5).
///
/// The `event` is the triggering name (for example `"click"`); the `handler`
/// is the command or action name that flows up to the host when the event
/// fires. Both are plain names, so this type is fully hashable and derives
/// `Eq`/`Hash`.
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
