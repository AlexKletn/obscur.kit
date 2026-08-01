#![doc = include_str!("undo_unit.doc/_module.md")]

use crate::selection::TextSelection;

use super::EditTransaction;

#[doc = include_str!("undo_unit.doc/UndoKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UndoKind {
    #[doc = include_str!("undo_unit.doc/Typing.md")]
    Typing,
    #[doc = include_str!("undo_unit.doc/Deletion.md")]
    Deletion,
    #[doc = include_str!("undo_unit.doc/Composition.md")]
    Composition,
    #[doc = include_str!("undo_unit.doc/Paste.md")]
    Paste,
    #[doc = include_str!("undo_unit.doc/Other.md")]
    Other,
}

impl UndoKind {
    #[doc = include_str!("undo_unit.doc/is_coalescing.md")]
    #[must_use]
    pub const fn is_coalescing(&self) -> bool {
        matches!(self, Self::Typing | Self::Deletion)
    }
}

#[doc = include_str!("undo_unit.doc/UndoUnit.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct UndoUnit {
    #[doc = include_str!("undo_unit.doc/edits.md")]
    edits: Vec<EditTransaction>,
    #[doc = include_str!("undo_unit.doc/kind.md")]
    kind: UndoKind,
}

impl UndoUnit {
    #[doc = include_str!("undo_unit.doc/new.md")]
    #[must_use]
    pub const fn new(kind: UndoKind) -> Self {
        Self {
            edits: Vec::new(),
            kind,
        }
    }

    #[doc = include_str!("undo_unit.doc/single.md")]
    #[must_use]
    pub fn single(edit: EditTransaction, kind: UndoKind) -> Self {
        Self {
            edits: vec![edit],
            kind,
        }
    }

    #[doc = include_str!("undo_unit.doc/push.md")]
    pub fn push(&mut self, edit: EditTransaction) {
        self.edits.push(edit);
    }

    #[doc = include_str!("undo_unit.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.edits.is_empty()
    }

    #[doc = include_str!("undo_unit.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.edits.len()
    }

    #[doc = include_str!("undo_unit.doc/edits_2.md")]
    #[must_use]
    pub fn edits(&self) -> &[EditTransaction] {
        &self.edits
    }

    #[doc = include_str!("undo_unit.doc/kind_2.md")]
    #[must_use]
    pub const fn kind(&self) -> UndoKind {
        self.kind
    }

    #[doc = include_str!("undo_unit.doc/can_coalesce_with.md")]
    #[must_use]
    pub const fn can_coalesce_with(&self, other_kind: UndoKind) -> bool {
        self.kind.is_coalescing() && self.kind_eq(other_kind)
    }

    #[doc = include_str!("undo_unit.doc/invert.md")]
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

    #[doc = include_str!("undo_unit.doc/selection_before.md")]
    #[must_use]
    pub fn selection_before(&self) -> Option<TextSelection> {
        self.edits.first().map(EditTransaction::selection_before)
    }

    #[doc = include_str!("undo_unit.doc/selection_after.md")]
    #[must_use]
    pub fn selection_after(&self) -> Option<TextSelection> {
        self.edits.last().map(EditTransaction::selection_after)
    }

    #[doc = include_str!("undo_unit.doc/kind_eq.md")]
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
