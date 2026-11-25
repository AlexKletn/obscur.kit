What produced an [`UndoUnit`](super::UndoUnit), governing whether consecutive units coalesce
(spec §5.11 undo primitives; §5.12).

The undo stack merges runs of the same fine-grained activity so a burst of
typing or deletion undoes as one step, while structural edits stay atomic. A
[`Composition`](Self::Composition) is never merged: per spec §5.12 "one
composition is one undo unit".
