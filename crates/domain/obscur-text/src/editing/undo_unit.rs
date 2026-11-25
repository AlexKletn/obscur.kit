//! A coalescable group of transactions that undo/redo as one step
//! (spec §5.11 undo primitives; §5.12 IME — one composition is one undo unit).

use crate::selection::TextSelection;

use super::EditTransaction;

use super::UndoKind;

#[doc = include_str!("../../docs/api/editing/undo_unit.doc/UndoUnit.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct UndoUnit {
    /// The edits in this unit, in the order they were applied.
    edits: Vec<EditTransaction>,
    /// The activity that produced the unit; governs coalescing.
    kind: UndoKind,
}

impl UndoUnit {
    /// Creates an empty unit of the given `kind`.
    #[must_use]
    pub const fn new(kind: UndoKind) -> Self {
        Self {
            edits: Vec::new(),
            kind,
        }
    }

    /// Creates a unit of `kind` holding the single edit `edit`.
    #[must_use]
    pub fn single(edit: EditTransaction, kind: UndoKind) -> Self {
        Self {
            edits: vec![edit],
            kind,
        }
    }

    /// Appends `edit` to this unit (as the newest edit in application order).
    pub fn push(&mut self, edit: EditTransaction) {
        self.edits.push(edit);
    }

    /// Returns whether the unit holds no edits.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.edits.is_empty()
    }

    /// Returns the number of edits in the unit.
    #[must_use]
    pub fn len(&self) -> usize {
        self.edits.len()
    }

    /// Returns the edits in application order.
    #[must_use]
    pub fn edits(&self) -> &[EditTransaction] {
        &self.edits
    }

    /// Returns the kind that produced this unit.
    #[must_use]
    pub const fn kind(&self) -> UndoKind {
        self.kind
    }

    #[doc = include_str!("../../docs/api/editing/undo_unit.doc/can_coalesce_with.md")]
    #[must_use]
    pub const fn can_coalesce_with(&self, other_kind: UndoKind) -> bool {
        self.kind.is_coalescing() && self.kind_eq(other_kind)
    }

    #[doc = include_str!("../../docs/api/editing/undo_unit.doc/invert.md")]
    #[must_use]
    pub fn invert(&self) -> UndoUnit {
        UndoUnit {
            edits: self
                .edits
                .iter()
                .rev()
                .map(EditTransaction::invert)
                .collect(),
            kind: self.kind,
        }
    }

    /// Returns the selection recorded before the unit's first edit, or `None`
    /// when the unit is empty.
    #[must_use]
    pub fn selection_before(&self) -> Option<TextSelection> {
        self.edits.first().map(EditTransaction::selection_before)
    }

    /// Returns the selection recorded after the unit's last edit, or `None` when
    /// the unit is empty.
    #[must_use]
    pub fn selection_after(&self) -> Option<TextSelection> {
        self.edits.last().map(EditTransaction::selection_after)
    }

    /// Const-friendly equality of two [`UndoKind`]s.
    const fn kind_eq(&self, other: UndoKind) -> bool {
        matches!(
            (self.kind, other),
            (UndoKind::Typing, UndoKind::Typing)
                | (UndoKind::Deletion, UndoKind::Deletion)
                | (UndoKind::Composition, UndoKind::Composition)
                | (UndoKind::Paste, UndoKind::Paste)
                | (UndoKind::Other, UndoKind::Other)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::TextRange;

    fn edit(start: usize, removed: &str, inserted: &str) -> EditTransaction {
        let end = start + removed.len();
        EditTransaction::new(
            TextRange::new(start, end),
            removed,
            inserted,
            TextSelection::caret(start),
            TextSelection::caret(start + inserted.len()),
        )
    }

    #[test]
    fn push_len_and_empty() {
        let mut u = UndoUnit::new(UndoKind::Typing);
        assert!(u.is_empty());
        u.push(edit(0, "", "a"));
        u.push(edit(1, "", "b"));
        assert_eq!(u.len(), 2);
        assert!(!u.is_empty());
        assert_eq!(u.edits().len(), 2);
    }

    #[test]
    fn coalesce_rules() {
        let u = UndoUnit::single(edit(0, "", "a"), UndoKind::Typing);
        assert!(u.can_coalesce_with(UndoKind::Typing));
        assert!(!u.can_coalesce_with(UndoKind::Deletion));
        assert!(!u.can_coalesce_with(UndoKind::Composition));

        let d = UndoUnit::single(edit(0, "a", ""), UndoKind::Deletion);
        assert!(d.can_coalesce_with(UndoKind::Deletion));
        assert!(!d.can_coalesce_with(UndoKind::Typing));

        // Atomic kinds never coalesce, even with themselves.
        let c = UndoUnit::single(edit(0, "", "x"), UndoKind::Composition);
        assert!(!c.can_coalesce_with(UndoKind::Composition));
        assert!(
            !UndoUnit::single(edit(0, "", "x"), UndoKind::Paste).can_coalesce_with(UndoKind::Paste)
        );
        assert!(
            !UndoUnit::single(edit(0, "", "x"), UndoKind::Other).can_coalesce_with(UndoKind::Other)
        );
    }

    #[test]
    fn invert_reverses_order_and_inverts_each() {
        let mut u = UndoUnit::new(UndoKind::Typing);
        let e0 = edit(0, "", "a");
        let e1 = edit(1, "", "b");
        u.push(e0.clone());
        u.push(e1.clone());

        let inv = u.invert();
        assert_eq!(inv.kind(), UndoKind::Typing);
        // Reversed order: e1's inverse first, then e0's.
        assert_eq!(inv.edits()[0], e1.invert());
        assert_eq!(inv.edits()[1], e0.invert());
    }

    #[test]
    fn selections_from_ends() {
        let mut u = UndoUnit::new(UndoKind::Typing);
        assert_eq!(u.selection_before(), None);
        assert_eq!(u.selection_after(), None);
        u.push(edit(0, "", "a")); // before caret(0), after caret(1)
        u.push(edit(1, "", "b")); // before caret(1), after caret(2)
        assert_eq!(u.selection_before(), Some(TextSelection::caret(0)));
        assert_eq!(u.selection_after(), Some(TextSelection::caret(2)));
    }
}
