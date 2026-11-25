use obscur_identity::InternedSymbol;
use obscur_interaction::command::{Command, CommandState};

#[doc = include_str!("../../docs/api/menu/menu.doc/MenuItem.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    pub(super) label: Box<str>,
    pub(super) command: InternedSymbol,
    pub(super) enabled: bool,
}

impl MenuItem {
    /// Creates an enabled item showing `label` that fires `command` when
    /// activated.
    #[must_use]
    pub fn new(label: impl Into<Box<str>>, command: InternedSymbol) -> Self {
        Self {
            label: label.into(),
            command,
            enabled: true,
        }
    }

    /// Marks the item disabled, returning it (fluent builder).
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    #[doc = include_str!("../../docs/api/menu/menu.doc/from_command.md")]
    #[must_use]
    pub fn from_command(command: &Command, state: CommandState) -> Self {
        Self {
            label: command.title().into(),
            command: command.id(),
            enabled: state.is_enabled(),
        }
    }

    /// Returns the item label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the command fired when the item is activated.
    #[must_use]
    pub const fn command(&self) -> InternedSymbol {
        self.command
    }

    /// Returns whether the item is enabled.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }
}
