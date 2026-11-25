//! `ViewEventBinding`: binds an event to an action that flows up.

use obscur_identity::InternedSymbol;

#[doc = include_str!("../../docs/api/view/view_event_binding.doc/ViewEventBinding.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ViewEventBinding {
    /// Interned name of the event this binding listens for.
    event: InternedSymbol,
    /// Interned handle of the semantic command invoked when the event fires.
    /// This is the action that flows *up* to the runtime.
    command: InternedSymbol,
}

impl ViewEventBinding {
    /// Binds `event` to the `command` it should invoke.
    #[must_use]
    pub const fn new(event: InternedSymbol, command: InternedSymbol) -> Self {
        Self { event, command }
    }

    /// Returns the interned event name.
    #[must_use]
    pub const fn event(&self) -> InternedSymbol {
        self.event
    }

    /// Returns the interned command invoked when the event fires.
    #[must_use]
    pub const fn command(&self) -> InternedSymbol {
        self.command
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_return_fields() {
        let event = InternedSymbol::from_raw(1);
        let command = InternedSymbol::from_raw(2);
        let binding = ViewEventBinding::new(event, command);
        assert_eq!(binding.event(), event);
        assert_eq!(binding.command(), command);
    }
}
