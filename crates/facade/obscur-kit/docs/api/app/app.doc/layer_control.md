A handle to the live desktop surface, for reconfiguring or closing it from a
handler.

The surface belongs to the loop, and a command handler has no reference to it.
This is the channel: a handler leaves a request, the loop applies it on its next
turn. Take it before [`run`](Self::run) — afterwards there is no `App` left to
ask.

Cloning gives another view of the same surface, never a second one.
