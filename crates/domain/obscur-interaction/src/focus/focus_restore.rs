#![doc = include_str!("focus_restore.doc/_module.md")]

use obscur_identity::NodeId;

#[doc = include_str!("focus_restore.doc/FocusRestore.md")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FocusRestore {
    stack: Vec<Option<NodeId>>,
}

impl FocusRestore {
    #[doc = include_str!("focus_restore.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    #[doc = include_str!("focus_restore.doc/save.md")]
    pub fn save(&mut self, previously_focused: Option<NodeId>) {
        self.stack.push(previously_focused);
    }

    #[doc = include_str!("focus_restore.doc/restore.md")]
    pub fn restore(&mut self) -> Option<NodeId> {
        self.stack.pop().flatten()
    }

    #[doc = include_str!("focus_restore.doc/peek.md")]
    #[must_use]
    pub fn peek(&self) -> Option<NodeId> {
        self.stack.last().copied().flatten()
    }

    #[doc = include_str!("focus_restore.doc/clear.md")]
    pub fn clear(&mut self) {
        self.stack.clear();
    }

    #[doc = include_str!("focus_restore.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    #[doc = include_str!("focus_restore.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    #[doc = include_str!("focus_restore.doc/depth.md")]
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
        r.save(Some(node(1)));
        r.save(Some(node(2)));
        r.save(Some(node(3)));
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
        r.save(Some(node(1)));
        r.save(Some(node(2)));
        r.clear();
        assert!(r.is_empty());
        assert_eq!(r.peek(), None);
    }

    /// A scope opening over an unfocused surface still owes a frame. Without
    /// one, the close that follows pops the *enclosing* scope's saved node and
    /// hands focus to a control the user never left — which is why a frame is
    /// `Option<NodeId>` and not `NodeId`.
    #[test]
    fn a_scope_that_displaced_nothing_still_pushes_a_frame() {
        let mut r = FocusRestore::new();
        r.save(Some(node(1))); // an outer scope, over a focused control
        r.save(None); // an inner one, over nothing
        assert_eq!(r.depth(), 2);
        assert_eq!(r.restore(), None, "the inner scope restores nothing");
        assert_eq!(
            r.restore(),
            Some(node(1)),
            "and the outer one still gets its own node back"
        );
        assert!(r.is_empty());
    }
}
