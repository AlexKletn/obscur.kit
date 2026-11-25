//! The [`CommandRouter`] shortcut-to-command binding table (spec §6.7).

use obscur_identity::InternedSymbol;

use crate::command::{LogicalKey, Modifiers, Shortcut};

#[doc = include_str!("../../docs/api/command/command_router.doc/CommandRouter.md")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandRouter {
    /// Ordered shortcut→command-symbol bindings; index order is priority.
    bindings: Vec<(Shortcut, InternedSymbol)>,
}

impl CommandRouter {
    /// Creates an empty router with no bindings.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    #[doc = include_str!("../../docs/api/command/command_router.doc/bind.md")]
    pub fn bind(&mut self, shortcut: Shortcut, command: InternedSymbol) {
        self.bindings.push((shortcut, command));
    }

    #[doc = include_str!("../../docs/api/command/command_router.doc/resolve.md")]
    #[must_use]
    pub fn resolve(&self, key: LogicalKey, modifiers: Modifiers) -> Option<InternedSymbol> {
        self.bindings
            .iter()
            .find(|(shortcut, _)| shortcut.matches(key, modifiers))
            .map(|(_, command)| *command)
    }

    #[doc = include_str!("../../docs/api/command/command_router.doc/unbind.md")]
    pub fn unbind(&mut self, shortcut: &Shortcut) -> bool {
        if let Some(index) = self.bindings.iter().position(|(sc, _)| sc == shortcut) {
            self.bindings.remove(index);
            true
        } else {
            false
        }
    }

    /// Returns every shortcut bound to `command`, in registration order.
    #[must_use]
    pub fn bindings_for(&self, command: InternedSymbol) -> Vec<Shortcut> {
        self.bindings
            .iter()
            .filter(|(_, bound)| *bound == command)
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

    /// App-owned command symbols the bindings resolve to.
    const CMD_SAVE: InternedSymbol = InternedSymbol::from_raw(0x5A7E);
    const CMD_SAVE_SHADOWED: InternedSymbol = InternedSymbol::from_raw(0x5A70);
    const CMD_OPEN: InternedSymbol = InternedSymbol::from_raw(0x0BE1);

    fn ctrl(c: char) -> Shortcut {
        Shortcut::new(LogicalKey::character(c)).with(Modifier::Ctrl)
    }

    #[test]
    fn bind_and_resolve_first_match_wins() {
        let mut router = CommandRouter::new();
        assert!(router.is_empty());

        router.bind(ctrl('s'), CMD_SAVE);
        router.bind(ctrl('s'), CMD_SAVE_SHADOWED);
        router.bind(ctrl('o'), CMD_OPEN);

        assert_eq!(router.len(), 3);
        let ctrl_mod = Modifiers::new().with(Modifier::Ctrl);
        assert_eq!(
            router.resolve(LogicalKey::character('s'), ctrl_mod),
            Some(CMD_SAVE)
        );
        assert_eq!(
            router.resolve(LogicalKey::character('o'), ctrl_mod),
            Some(CMD_OPEN)
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
        router.bind(ctrl('s'), CMD_SAVE);
        router.bind(ctrl('s'), CMD_SAVE_SHADOWED);

        assert!(router.unbind(&ctrl('s')));
        assert_eq!(router.len(), 1);
        let ctrl_mod = Modifiers::new().with(Modifier::Ctrl);
        // The shadowed binding is now the winner.
        assert_eq!(
            router.resolve(LogicalKey::character('s'), ctrl_mod),
            Some(CMD_SAVE_SHADOWED)
        );
        // Unbinding a missing shortcut reports false.
        assert!(!router.unbind(&ctrl('z')));
    }

    #[test]
    fn bindings_for_returns_all_shortcuts() {
        let mut router = CommandRouter::new();
        router.bind(ctrl('s'), CMD_SAVE);
        router.bind(Shortcut::new(LogicalKey::character('w')), CMD_SAVE);
        router.bind(ctrl('o'), CMD_OPEN);

        let saves = router.bindings_for(CMD_SAVE);
        assert_eq!(saves.len(), 2);
        assert!(saves.contains(&ctrl('s')));
        assert!(saves.contains(&Shortcut::new(LogicalKey::character('w'))));
        assert!(router.bindings_for(CMD_OPEN).len() == 1);
        assert!(
            router
                .bindings_for(InternedSymbol::from_raw(0xDEAD))
                .is_empty()
        );
    }

    #[test]
    fn clear_empties_the_router() {
        let mut router = CommandRouter::new();
        router.bind(ctrl('s'), CMD_SAVE);
        router.clear();
        assert!(router.is_empty());
        assert_eq!(router.len(), 0);
        assert_eq!(router, CommandRouter::default());
    }
}
