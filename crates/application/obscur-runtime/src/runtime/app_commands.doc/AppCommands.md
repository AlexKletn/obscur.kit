The **application** level of §2.6's command routing chain — its last link.

A [`UiRuntime`](super::UiRuntime) is one *window surface*: it owns one tree,
one viewport and one focus. So a handler registered on the runtime itself
([`on_command`](super::UiRuntime::on_command)) is a **window**-level handler
— link 4 — and cannot also serve as §2.6's *application* fallback, which by
definition outlives and spans windows.

`AppCommands` is that level: a registry held behind a shared handle, so the
**same** fallback can be installed on every window's runtime via
[`set_app_commands`](super::UiRuntime::set_app_commands). Cloning an
`AppCommands` shares the underlying table rather than copying it — a command
registered once is then the fallback for every window that shares it, which
is what makes the window and application links genuinely distinct rather than
two maps consulted in sequence.

Handlers take either shape [`IntoCommandHandler`] accepts — `FnMut()` or
`FnMut(&mut UiEvent)` — and are not `Send`, matching the rest of the runtime's
single-threaded handler model.
