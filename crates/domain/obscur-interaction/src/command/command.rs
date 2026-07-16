//! The [`Command`] unified action descriptor (spec §6.7).

use obscur_identity::InternedSymbol;

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
///
/// # Why the id is an `InternedSymbol`
///
/// The id is the **app-owned command symbol** every live surface already routes:
/// the symbol [`CommandRouter`](crate::command::CommandRouter) resolves a
/// shortcut to, the symbol a `MenuItem` fires, and the symbol the runtime
/// registers a handler for. It is deliberately *not* a string. This crate has no
/// string→symbol interner and cannot reach one — [`InternedSymbol::from_raw`] is
/// the type's only constructor, and the workspace's single interner belongs to
/// the CSS compiler (a `frontend` crate this `domain` crate may not depend on),
/// mints sequential ids that would collide with app-assigned symbols, and
/// documents itself as not shared. A string-keyed id could therefore never name
/// the thing that routes: it would be a parallel identity no surface could act
/// on. Keying on the routed symbol is what lets one `Command` drive every
/// surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// The stable command symbol linking every surface to this action; the same
    /// symbol shortcuts resolve to and handlers are registered for.
    id: InternedSymbol,
    /// The human-readable title shown in menus, palettes, and toolbars.
    title: Box<str>,
    /// The keyboard shortcut, if any, that triggers this command.
    shortcut: Option<Shortcut>,
    /// The icon name/id, if any, for toolbar and menu presentation.
    icon: Option<Box<str>>,
}

impl Command {
    /// Creates a command with the app-owned command symbol `id` and
    /// human-readable `title`, with no shortcut or icon.
    #[must_use]
    pub fn new(id: InternedSymbol, title: impl Into<Box<str>>) -> Self {
        Self {
            id,
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

    /// Returns the stable command symbol every surface routes.
    #[must_use]
    pub const fn id(&self) -> InternedSymbol {
        self.id
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
