Services the backend's queued work each turn, then acts on the domain
window events it produced.

Forwarding to the handler here is not optional: it is what applies any
operation the host issued on the backend between turns (the
`close_window` below, property updates, later `create_window` calls).
