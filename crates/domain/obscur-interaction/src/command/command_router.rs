//! The [`CommandRouter`] shortcut-to-command binding table (spec §6.7).

use crate::command::{LogicalKey, Modifiers, Shortcut};

/// Maps shortcuts to command ids and resolves key input (spec §6.7).
///
/// A `CommandRouter` holds an ordered list of shortcut→command-id bindings.
/// Registration order is priority: [`resolve`](CommandRouter::resolve) returns
/// the command id of the *first* binding whose shortcut matches the key input,
/// so earlier bindings shadow later ones. The router stores only ids; it does
/// not own the [`Command`](crate::command::Command)s themselves.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommandRouter {
    /// Ordered shortcut→command-id bindings; index order is priority.
    bindings: Vec<(Shortcut, Box<str>)>,
}

impl CommandRouter {
    /// Creates an empty router with no bindings.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    /// Binds `shortcut` to `command_id`, appending it at the lowest priority.
    ///
    /// Duplicate shortcuts are allowed; the earliest-registered one wins during
    /// [`resolve`](CommandRouter::resolve).
    pub fn bind(&mut self, shortcut: Shortcut, command_id: impl Into<Box<str>>) {
        self.bindings.push((shortcut, command_id.into()));
    }

    /// Resolves key input to a command id.
    ///
    /// Returns the id of the first bound command whose shortcut matches `key`
    /// with exactly `modifiers` held, in registration order (first match wins),
    /// or `None` if nothing matches.
    #[must_use]
    pub fn resolve(&self, key: LogicalKey, modifiers: Modifiers) -> Option<&str> {
        self.bindings
            .iter()
            .find(|(shortcut, _)| shortcut.matches(key, modifiers))
            .map(|(_, id)| id.as_ref())
    }

    /// Removes the first binding with this exact `shortcut`.
    ///
    /// Returns whether a binding was removed.
    pub fn unbind(&mut self, shortcut: &Shortcut) -> bool {
        if let Some(index) = self.bindings.iter().position(|(sc, _)| sc == shortcut) {
            self.bindings.remove(index);
            true
        } else {
            false
        }
    }

    /// Returns every shortcut bound to `command_id`, in registration order.
    #[must_use]
    pub fn bindings_for(&self, command_id: &str) -> Vec<Shortcut> {
        self.bindings
            .iter()
            .filter(|(_, id)| id.as_ref() == command_id)
            .map(|(sc, _)| *sc)
            .collect()
    }

    /// Returns the number of bindings.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Returns whether the router has no bindings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Removes every binding.
    pub fn clear(&mut self) {
        self.bindings.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Modifier;

    fn ctrl(c: char) -> Shortcut {
        Shortcut::new(LogicalKey::character(c)).with(Modifier::Ctrl)
    }

    #[test]
    fn bind_and_resolve_first_match_wins() {
        let mut router = CommandRouter::new();
        assert!(router.is_empty());

        router.bind(ctrl('s'), "file.save");
        router.bind(ctrl('s'), "file.save_shadowed");
        router.bind(ctrl('o'), "file.open");

        assert_eq!(router.len(), 3);
        let ctrl_mod = Modifiers::new().with(Modifier::Ctrl);
        assert_eq!(
            router.resolve(LogicalKey::character('s'), ctrl_mod),
            Some("file.save")
        );
        assert_eq!(
            router.resolve(LogicalKey::character('o'), ctrl_mod),
            Some("file.open")
        );
        // No match: missing modifier.
        assert_eq!(
            router.resolve(LogicalKey::character('s'), Modifiers::new()),
            None
        );
        // No match: unbound key.
        assert_eq!(router.resolve(LogicalKey::character('x'), ctrl_mod), None);
    }

    #[test]
    fn unbind_removes_first_matching_shortcut() {
        let mut router = CommandRouter::new();
        router.bind(ctrl('s'), "file.save");
        router.bind(ctrl('s'), "file.save_shadowed");

        assert!(router.unbind(&ctrl('s')));
        assert_eq!(router.len(), 1);
        let ctrl_mod = Modifiers::new().with(Modifier::Ctrl);
        // The shadowed binding is now the winner.
        assert_eq!(
            router.resolve(LogicalKey::character('s'), ctrl_mod),
            Some("file.save_shadowed")
        );
        // Unbinding a missing shortcut reports false.
        assert!(!router.unbind(&ctrl('z')));
    }

    #[test]
    fn bindings_for_returns_all_shortcuts() {
        let mut router = CommandRouter::new();
        router.bind(ctrl('s'), "file.save");
        router.bind(Shortcut::new(LogicalKey::character('w')), "file.save");
        router.bind(ctrl('o'), "file.open");

        let saves = router.bindings_for("file.save");
        assert_eq!(saves.len(), 2);
        assert!(saves.contains(&ctrl('s')));
        assert!(saves.contains(&Shortcut::new(LogicalKey::character('w'))));
        assert!(router.bindings_for("missing").is_empty());
    }

    #[test]
    fn clear_empties_the_router() {
        let mut router = CommandRouter::new();
        router.bind(ctrl('s'), "file.save");
        router.clear();
        assert!(router.is_empty());
        assert_eq!(router.len(), 0);
        assert_eq!(router, CommandRouter::default());
    }
}
