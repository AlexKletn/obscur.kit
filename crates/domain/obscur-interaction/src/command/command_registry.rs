#![doc = include_str!("command_registry.doc/_module.md")]

use std::collections::HashMap;

use obscur_identity::InternedSymbol;

use crate::command::{Command, CommandState};

#[doc = include_str!("command_registry.doc/CommandRegistry.md")]
#[derive(Debug, Clone, Default)]
pub struct CommandRegistry {
    #[doc = include_str!("command_registry.doc/entries.md")]
    entries: HashMap<InternedSymbol, (Command, CommandState)>,
}

impl CommandRegistry {
    #[doc = include_str!("command_registry.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    #[doc = include_str!("command_registry.doc/register.md")]
    pub fn register(&mut self, command: Command) -> Option<Command> {
        let id = command.id();
        self.entries
            .insert(id, (command, CommandState::new()))
            .map(|(previous, _)| previous)
    }

    #[doc = include_str!("command_registry.doc/get.md")]
    #[must_use]
    pub fn get(&self, id: InternedSymbol) -> Option<&Command> {
        self.entries.get(&id).map(|(command, _)| command)
    }

    #[doc = include_str!("command_registry.doc/state.md")]
    #[must_use]
    pub fn state(&self, id: InternedSymbol) -> Option<CommandState> {
        self.entries.get(&id).map(|&(_, state)| state)
    }

    #[doc = include_str!("command_registry.doc/set_state.md")]
    pub fn set_state(&mut self, id: InternedSymbol, state: CommandState) -> bool {
        match self.entries.get_mut(&id) {
            Some((_, slot)) => {
                *slot = state;
                true
            }
            None => false,
        }
    }

    #[doc = include_str!("command_registry.doc/permits_dispatch.md")]
    #[must_use]
    pub fn permits_dispatch(&self, id: InternedSymbol) -> bool {
        self.state(id)
            .is_none_or(|state: CommandState| state.is_actionable())
    }

    #[doc = include_str!("command_registry.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, id: InternedSymbol) -> bool {
        self.entries.contains_key(&id)
    }

    #[doc = include_str!("command_registry.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[doc = include_str!("command_registry.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{LogicalKey, Modifier, Shortcut};

    const CMD_SAVE: InternedSymbol = InternedSymbol::from_raw(0x5A7E);
    const CMD_OPEN: InternedSymbol = InternedSymbol::from_raw(0x0BE1);
    const CMD_UNREGISTERED: InternedSymbol = InternedSymbol::from_raw(0xDEAD);

    #[test]
    fn register_keys_the_command_on_its_own_symbol() {
        let mut registry = CommandRegistry::new();
        assert!(registry.is_empty());

        let save = Shortcut::new(LogicalKey::character('s')).with(Modifier::Ctrl);
        assert!(
            registry
                .register(Command::new(CMD_SAVE, "Save").with_shortcut(save))
                .is_none()
        );

        assert_eq!(registry.len(), 1);
        assert!(registry.contains(CMD_SAVE));
        // The registry is reachable by the SAME symbol a shortcut resolves to.
        let found = registry.get(CMD_SAVE).expect("registered");
        assert_eq!(found.id(), CMD_SAVE);
        assert_eq!(found.title(), "Save");
        assert_eq!(found.shortcut(), Some(save));
        assert_eq!(registry.get(CMD_UNREGISTERED), None);
    }

    #[test]
    fn register_replaces_descriptor_and_resets_state() {
        let mut registry = CommandRegistry::new();
        registry.register(Command::new(CMD_SAVE, "Save"));
        assert!(registry.set_state(CMD_SAVE, CommandState::new().enabled(false)));

        let previous = registry.register(Command::new(CMD_SAVE, "Save As"));
        assert_eq!(previous.map(|c| c.title().to_owned()), Some("Save".into()));
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.get(CMD_SAVE).map(Command::title), Some("Save As"));
        // The fresh descriptor arrives with the default (live) state.
        assert_eq!(registry.state(CMD_SAVE), Some(CommandState::new()));
    }

    #[test]
    fn state_round_trips_and_is_none_when_unregistered() {
        let mut registry = CommandRegistry::new();
        registry.register(Command::new(CMD_SAVE, "Save"));

        assert_eq!(registry.state(CMD_SAVE), Some(CommandState::new()));
        assert_eq!(registry.state(CMD_UNREGISTERED), None);

        let checked = CommandState::new().with_checked(Some(true));
        assert!(registry.set_state(CMD_SAVE, checked));
        assert_eq!(registry.state(CMD_SAVE), Some(checked));

        // Setting state on an unregistered symbol reports false and stores nothing.
        assert!(!registry.set_state(CMD_UNREGISTERED, checked));
        assert_eq!(registry.state(CMD_UNREGISTERED), None);
    }

    #[test]
    fn permits_dispatch_gates_registered_commands_only() {
        let mut registry = CommandRegistry::new();
        registry.register(Command::new(CMD_SAVE, "Save"));
        registry.register(Command::new(CMD_OPEN, "Open"));

        // Registered + default state: permitted.
        assert!(registry.permits_dispatch(CMD_SAVE));
        // Unregistered: unconstrained, the registry gates nothing it never described.
        assert!(registry.permits_dispatch(CMD_UNREGISTERED));

        // Disabled and hidden are each independently inert.
        registry.set_state(CMD_SAVE, CommandState::new().enabled(false));
        assert!(!registry.permits_dispatch(CMD_SAVE));
        registry.set_state(CMD_SAVE, CommandState::new().visible(false));
        assert!(!registry.permits_dispatch(CMD_SAVE));

        // Gating one command leaves its neighbour alone.
        assert!(registry.permits_dispatch(CMD_OPEN));

        // Re-enabling restores it.
        registry.set_state(CMD_SAVE, CommandState::new());
        assert!(registry.permits_dispatch(CMD_SAVE));
    }
}
