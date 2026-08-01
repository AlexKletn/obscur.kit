#![doc = include_str!("view_event_binding.doc/_module.md")]

use obscur_identity::InternedSymbol;

#[doc = include_str!("view_event_binding.doc/ViewEventBinding.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ViewEventBinding {
    #[doc = include_str!("view_event_binding.doc/event.md")]
    event: InternedSymbol,
    #[doc = include_str!("view_event_binding.doc/command.md")]
    command: InternedSymbol,
}

impl ViewEventBinding {
    #[doc = include_str!("view_event_binding.doc/new.md")]
    #[must_use]
    pub const fn new(event: InternedSymbol, command: InternedSymbol) -> Self {
        Self { event, command }
    }

    #[doc = include_str!("view_event_binding.doc/event_2.md")]
    #[must_use]
    pub const fn event(&self) -> InternedSymbol {
        self.event
    }

    #[doc = include_str!("view_event_binding.doc/command_2.md")]
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
