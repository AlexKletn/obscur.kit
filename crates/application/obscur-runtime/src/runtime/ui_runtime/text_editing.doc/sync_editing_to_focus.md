Reconciles the editing session with the current focus: begins a fresh
session (seeded from the field's text, caret at the end) when focus is on
an editable the session does not already cover, and clears the session
when focus is not on any editable. Called whenever focus changes.

**Blur commits `change`.** When focus *leaves* the field the current
session covers (focus moving to a different editable, a non-editable, or
nothing), that is a **commit** in the HTML sense (WHATWG HTML §4.10.5.5: a
text field's `change` fires when it loses focus after its value changed).
So before re-seating/clearing the session this dispatches the outgoing
field's [`EVENT_CHANGE`](obscur_view::view::EVENT_CHANGE) command with the
committed text — once, and only if the value actually changed since focus
(see [`commit_editable_change`](Self::commit_editable_change)). Re-focusing
the *same* field is not a blur, so the session (and its baseline) persists
untouched.
