Queues `action` to run once against the runtime at the next
[`apply`](Self::apply).

The general escape hatch, for the runtime facilities this handle does not
wrap — `scroll_into_view`, `set_content_offset`, `open_focus_scope`,
`on_scope_command`, a `register_component`. It is a queued `FnOnce` rather
than a `&mut UiRuntime` accessor for the reason the type documents: at the
moment application code runs, the runtime is mid-frame. Here it is not, and
the borrow lasts exactly the call — `action` cannot store it.

Queue it from anywhere, including a command handler. It runs after that
handler has returned, so it can safely do things a handler must not: mutate
the tree's registrations, replace the root view, or dispatch a command.
