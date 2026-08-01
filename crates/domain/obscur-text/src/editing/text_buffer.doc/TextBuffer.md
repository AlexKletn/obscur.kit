The pure editing engine: UTF-8 text, one selection, and local undo/redo
(spec §5.11 editing engine).

A `TextBuffer` is a deterministic state machine. [`apply`](Self::apply)
consumes an [`EditCommand`] and updates the text and selection, pushing an
[`UndoUnit`] for every command that actually changes text; [`undo`](Self::undo)
and [`redo`](Self::redo) walk those units, restoring both the text and the
recorded selection. Runs of typing and of deletion coalesce into a single
undo step (see [`UndoKind`]); every other kind of edit is atomic.

The buffer holds exactly one [`TextSelection`]. Multi-selection is a
higher-level concern layered on top of this engine.
