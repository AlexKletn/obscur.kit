//! The [`FocusRestore`] saved-focus stack for scope enter/exit
//! (spec §7.1 focus).

use obscur_identity::NodeId;

/// A LIFO stack of saved focus for scope enter/exit (spec §7.1).
///
/// When a focus scope (a dialog or menu) opens, the runtime saves the node that
/// was focused just before it. When the scope closes, that node is restored.
/// Because scopes nest, saves and restores are last-in/first-out: closing scopes
/// restore focus in the reverse order they were opened.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FocusRestore {
    stack: Vec<NodeId>,
}

impl FocusRestore {
    /// Creates an empty restore stack.
    #[must_use]
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Saves `previously_focused` as the focus to restore when the current scope
    /// closes, pushing it onto the stack.
    pub fn save(&mut self, previously_focused: NodeId) {
        self.stack.push(previously_focused);
    }

    /// Restores the most recently saved focus, removing it from the stack.
    pub fn restore(&mut self) -> Option<NodeId> {
        self.stack.pop()
    }

    /// Returns the most recently saved focus without removing it.
    #[must_use]
    pub fn peek(&self) -> Option<NodeId> {
        self.stack.last().copied()
    }

    /// Discards all saved focus.
    pub fn clear(&mut self) {
        self.stack.clear();
    }

    /// Returns the number of saved entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Returns whether nothing is saved.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Returns the current nesting depth (number of saved entries).
    #[must_use]
    pub fn depth(&self) -> usize {
        self.stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn new_is_empty() {
        let r = FocusRestore::new();
        assert!(r.is_empty());
        assert_eq!(r.depth(), 0);
        assert_eq!(r, FocusRestore::default());
    }

    #[test]
    fn save_and_restore_is_lifo() {
        let mut r = FocusRestore::new();
        r.save(node(1));
        r.save(node(2));
        r.save(node(3));
        assert_eq!(r.depth(), 3);
        assert_eq!(r.peek(), Some(node(3)));
        assert_eq!(r.restore(), Some(node(3)));
        assert_eq!(r.restore(), Some(node(2)));
        assert_eq!(r.restore(), Some(node(1)));
        assert_eq!(r.restore(), None);
        assert!(r.is_empty());
    }

    #[test]
    fn clear_discards_all() {
        let mut r = FocusRestore::new();
        r.save(node(1));
        r.save(node(2));
        r.clear();
        assert!(r.is_empty());
        assert_eq!(r.peek(), None);
    }
}
