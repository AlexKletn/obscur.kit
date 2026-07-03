//! The [`Command`] unified action descriptor (spec §6.7).

use crate::command::Shortcut;

/// A unified action descriptor (spec §6.7).
///
/// A `Command` is the single declarative descriptor of an action. The same
/// command drives menus, toolbars, the command palette, keyboard shortcuts, and
/// accessibility — a stable [`id`](Command::id) links every surface to one
/// action. This type carries only the static description (identity, title,
/// shortcut, icon); the dynamic runtime state (enabled/checked/visible) lives in
/// [`CommandState`](crate::command::CommandState), and dispatch lives in a
/// handler outside this pure-domain crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// The stable command id linking every surface to this action.
    id: Box<str>,
    /// The human-readable title shown in menus, palettes, and toolbars.
    title: Box<str>,
    /// The keyboard shortcut, if any, that triggers this command.
    shortcut: Option<Shortcut>,
    /// The icon name/id, if any, for toolbar and menu presentation.
    icon: Option<Box<str>>,
}

impl Command {
    /// Creates a command with a stable `id` and human-readable `title`, with no
    /// shortcut or icon.
    #[must_use]
    pub fn new(id: impl Into<Box<str>>, title: impl Into<Box<str>>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            shortcut: None,
            icon: None,
        }
    }

    /// Returns this command with its keyboard `shortcut` set.
    #[must_use]
    pub fn with_shortcut(mut self, shortcut: Shortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Returns this command with its `icon` name/id set.
    #[must_use]
    pub fn with_icon(mut self, icon: impl Into<Box<str>>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Returns the stable command id.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the human-readable title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the keyboard shortcut, if any.
    #[must_use]
    pub const fn shortcut(&self) -> Option<Shortcut> {
        self.shortcut
    }

    /// Returns the icon name/id, if any.
    #[must_use]
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{LogicalKey, Modifier};

    #[test]
    fn new_has_no_shortcut_or_icon() {
        let cmd = Command::new("file.save", "Save");
        assert_eq!(cmd.id(), "file.save");
        assert_eq!(cmd.title(), "Save");
        assert_eq!(cmd.shortcut(), None);
        assert_eq!(cmd.icon(), None);
    }

    #[test]
    fn builder_sets_shortcut_and_icon() {
        let sc = Shortcut::new(LogicalKey::character('s')).with(Modifier::Ctrl);
        let cmd = Command::new("file.save", "Save")
            .with_shortcut(sc)
            .with_icon("save");

        assert_eq!(cmd.shortcut(), Some(sc));
        assert_eq!(cmd.icon(), Some("save"));
    }
}
