The editing engine: a pure, deterministic text + selection state machine
(spec §5.11 editing engine).

[`TextBuffer`] owns a UTF-8 [`String`], a single [`TextSelection`], and the
local undo/redo stacks. It applies [`EditCommand`]s — resolving navigation,
deletion, insertion and replacement against the current text and selection —
and records each text change as a coalescable [`UndoUnit`] of invertible
[`EditTransaction`]s. Everything here is deterministic and layout-free:
`TextInput`, `TextArea`, and higher-level editors are built on this engine.

# UTF-8 correctness

All offsets are *byte* offsets that this engine keeps on UTF-8 character
boundaries. Untrusted offsets (from [`set_selection`](TextBuffer::set_selection)
or [`EditCommand::Replace`]) are clamped into range and snapped down to the
nearest character boundary before any slicing; the buffer never slices on a
non-boundary.

# Simplifications (adapter concerns)

This crate is pure-domain and has no text layout, so several motions are
approximated and are documented on their resolvers:

* **Grapheme** motion treats one `char` (Unicode scalar value) as one
  grapheme. True extended grapheme clusters (combining marks, ZWJ emoji,
  regional indicators) are a shaping/adapter concern; a layout-aware adapter
  can pre-resolve cluster boundaries and drive the buffer with explicit
  ranges via [`EditCommand::Replace`] / [`EditCommand::SetSelection`].
* **Word** motion is a simplified UAX-29 approximation: skip a run of
  whitespace then a run of non-whitespace in the travel direction.
* **Line** and **Paragraph** motion collapse to *logical* line boundaries
  (delimited by `'\n'`); true vertical/visual up-down motion needs layout and
  is left to an adapter.
