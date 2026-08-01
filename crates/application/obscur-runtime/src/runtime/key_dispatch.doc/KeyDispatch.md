The result of routing a keyboard event through
[`UiRuntime::handle_key`](super::UiRuntime::handle_key).

It is the keyboard analogue of
[`PointerDispatch`](super::PointerDispatch): it reports the focused
[`target`](Self::target) the key was routed to (if any), the
[`command`](Self::command) symbol the runtime's
[`CommandRouter`](obscur_interaction::command::CommandRouter) resolved the key
press to (if a bound shortcut matched — the command was also *dispatched*, so
its registered handler ran), and whether the key was
[`handled`](Self::is_handled) — i.e. reached a focused node.

When no node holds focus the key routes nowhere: `target` is `None`,
`command` is `None`, and `is_handled()` is `false` — the runtime's state is
unchanged and no frame is requested.
