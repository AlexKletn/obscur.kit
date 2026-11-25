A group of [`EditTransaction`]s that undo and redo as a single step (spec
§5.11 editing engine — local undo/redo primitives; §5.12).

One user-visible action can splice text in several places (multi-caret edit)
or accrete over time (a typing run), yet must undo atomically. An
[`UndoUnit`] bundles those transactions and tags them with an
[`UndoKind`](UndoKind) that decides whether the next action merges into this
unit or starts a new one. Undoing a unit applies its [`invert`](Self::invert);
redoing re-applies the original edits.
