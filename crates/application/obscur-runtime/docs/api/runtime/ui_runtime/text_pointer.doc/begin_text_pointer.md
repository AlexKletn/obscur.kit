On a pointer press: if it landed on the active editable, places the caret
at the click point and arms drag-selection anchored there (spec §5.11).

[`TextPointerSelection::press`] resolves what the press means against the
active editing session — a caret, a word (double click) or a line (triple
click) — and this seats the answer: every selection goes through the
standard non-mutating [`apply_edit`](Self::apply_edit) choke point (a frame,
no `on-input`), and a single click's anchor is armed on the pointer
sessions, which the resolver does not hold. A press on a non-editable (or
one whose text box is not laid out) clears any armed drag and leaves the
session byte-identical.
