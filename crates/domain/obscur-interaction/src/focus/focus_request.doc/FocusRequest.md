A request to move focus (spec §7.1).

A `FocusRequest` names the *target* of a focus move: a specific node, a
directional/tab step, the first or last tab stop, or clearing focus. It is a
pure command value; the focus state machine resolves it against the current
state and reports a [`FocusResult`](super::FocusResult). The motivating
[`FocusReason`] is supplied separately.
