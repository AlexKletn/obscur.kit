#![doc = include_str!("command.doc/_module.md")]

use obscur_identity::InternedSymbol;

use crate::command::Shortcut;

#[doc = include_str!("command.doc/Command.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    #[doc = include_str!("command.doc/id.md")]
    id: InternedSymbol,
    #[doc = include_str!("command.doc/title.md")]
    title: Box<str>,
    #[doc = include_str!("command.doc/shortcut.md")]
    shortcut: Option<Shortcut>,
    #[doc = include_str!("command.doc/icon.md")]
    icon: Option<Box<str>>,
}

impl Command {
    #[doc = include_str!("command.doc/new.md")]
    #[must_use]
    pub fn new(id: InternedSymbol, title: impl Into<Box<str>>) -> Self {
        Self {
            id,
            title: title.into(),
            shortcut: None,
            icon: None,
        }
    }

    #[doc = include_str!("command.doc/with_shortcut.md")]
    #[must_use]
    pub fn with_shortcut(mut self, shortcut: Shortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    #[doc = include_str!("command.doc/with_icon.md")]
    #[must_use]
    pub fn with_icon(mut self, icon: impl Into<Box<str>>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    #[doc = include_str!("command.doc/id_2.md")]
    #[must_use]
    pub const fn id(&self) -> InternedSymbol {
        self.id
    }

    #[doc = include_str!("command.doc/title_2.md")]
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[doc = include_str!("command.doc/shortcut_2.md")]
    #[must_use]
    pub const fn shortcut(&self) -> Option<Shortcut> {
        self.shortcut
    }

    #[doc = include_str!("command.doc/icon_2.md")]
    #[must_use]
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{LogicalKey, Modifier};

    /// The app-owned command symbol, as an application would define it.
    const CMD_SAVE: InternedSymbol = InternedSymbol::from_raw(0x5A7E);

    #[test]
    fn new_has_no_shortcut_or_icon() {
        let cmd = Command::new(CMD_SAVE, "Save");
        assert_eq!(cmd.id(), CMD_SAVE);
        assert_eq!(cmd.title(), "Save");
        assert_eq!(cmd.shortcut(), None);
        assert_eq!(cmd.icon(), None);
    }

    #[test]
    fn builder_sets_shortcut_and_icon() {
        let sc = Shortcut::new(LogicalKey::character('s')).with(Modifier::Ctrl);
        let cmd = Command::new(CMD_SAVE, "Save")
            .with_shortcut(sc)
            .with_icon("save");

        assert_eq!(cmd.shortcut(), Some(sc));
        assert_eq!(cmd.icon(), Some("save"));
    }
}
