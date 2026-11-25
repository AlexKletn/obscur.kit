What one key press resolved to for the focused editable — the answer
[`KeyEditResolver::resolve`] returns and the runtime acts on.

Every variant is a *decision*, never an effect: the resolver reads the field
and the session and says what the keystroke means, and the runtime applies it
through its own edit choke point. That is what keeps the keymap testable
without a scheduler, a handler registry or a frame.
