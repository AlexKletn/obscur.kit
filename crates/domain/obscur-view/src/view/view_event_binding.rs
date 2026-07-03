//! `ViewEventBinding`: binds an event to an action that flows up.

use obscur_identity::InternedSymbol;

/// Binds a typed event on a node to the semantic command it should invoke.
///
/// Events are typed and flow *up*: a node reports an event, and the bound
/// command is the typed action carried upward for the runtime to dispatch
/// (spec §2.3, §3.1). To keep the desired tree plain, comparable data, the
/// binding is fully decoupled from any runtime: it stores **no** closures,
/// `Box<dyn Fn>`, or `Rc` handles. Both the event and its target command are
/// referenced by [`InternedSymbol`]; the actual dispatch, capture/bubble
/// propagation, and closures live in `obscur-runtime`, not here.
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
