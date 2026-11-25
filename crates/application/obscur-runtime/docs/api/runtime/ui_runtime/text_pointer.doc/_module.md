The pointer's half of text selection: what a press on a text field does to the
caret, and what a drag with the button still down does to the selection.

The geometry — which glyph a point falls in, where a word or a line begins and
ends — is [`TextPointerSelection`](crate::runtime::TextPointerSelection)'s, and
it answers in [`TextPress`](crate::runtime::TextPress): ignore this, put a
collapsed caret here, or select this range. What is left for the runtime is the
part that entity is deliberately not allowed to do — apply the selection through
the ordinary edit path, and remember whether a drag is armed.

Both of those matter:

- **A selection is an edit.** Routing it through `apply_edit` rather than writing
  the session's selection directly is what makes a click ask for a frame, keeps
  the caret in view, and leaves `on-input` unfired — `SetSelection` is
  non-mutating, so the same tail does the right thing for a click that changes no
  text.
- **The armed drag is per-press state, and it lives with the pointer.** A press
  clears it first and re-arms it only for the single-click case, so a
  double-click that selected a word does not then extend by dragging; and
  because it is held by
  [`PointerSessions`](crate::runtime::PointerSessions), a lift or a cancel ends
  it with everything else that press owned, rather than through a teardown this
  file would otherwise have to be told to run.
