Dispatches a command and reports the whole outcome, not just whether a handler
claimed it.

[`dispatch_command_event`](Self::dispatch_command_event) is this with `.claimed`
taken off the end, and remains the shape almost every caller wants. This one
exists for the callers that must then decide whether to perform a default action
of their own — the keyboard path is the first, since a `keydown` listener's
`prevent_default` has to suppress the edit, the shortcut and the activation that
would otherwise follow.
