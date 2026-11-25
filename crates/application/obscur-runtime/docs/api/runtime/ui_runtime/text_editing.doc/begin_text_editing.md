Begins a fresh editing session on `node` seeded with `initial_text`
(caret at the end), replacing any existing session — the explicit,
app-driven way to seat the focused field's editable state (spec §5.11).

The runtime otherwise seeds a session automatically when focus lands on an
editable (see [`sync_editing_to_focus`](Self::sync_editing_to_focus)); an
app whose field carries its text as a *value property* rather than a text
node (e.g. [`Input`](../../obscur_widgets/input/struct.Input.html)) calls
this to seed the buffer with the real value.
