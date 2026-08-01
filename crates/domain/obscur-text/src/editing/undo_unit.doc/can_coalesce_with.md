Returns whether a new action of `other_kind` should merge into this unit
rather than open a fresh one.

True only when the kinds match *and* the kind is coalescing
([`Typing`](UndoKind::Typing) or [`Deletion`](UndoKind::Deletion)):
typing merges with typing and deletion with deletion, but never with each
other, and [`Composition`](UndoKind::Composition)/[`Paste`](UndoKind::Paste)/[`Other`](UndoKind::Other)
never merge.
