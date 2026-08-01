02 · Text Field: a live input beside a **read-only** one.

The read-only field is `Input::with_value(..).readonly(true)` (HTML `readonly`,
WHATWG HTML §attr-input-readonly): unlike a `disabled(true)` field it CAN be
focused and its "read only" text selected / copied — it just cannot be edited
(typing / Backspace / IME / paste are no-ops and its `change` never fires). The
text is its real value. It reuses `wf-field` (no distinct read-only styling: the
selector engine has no `:read-only` pseudo-class yet, so a read-only field looks
like a normal field here — an author would add a class for a muted look).
