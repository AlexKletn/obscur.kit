Whether dispatching `id` is permitted right now.

This is the policy every surface shares, stated once: a symbol with **no**
registered command is unconstrained (`true`) — the registry describes
commands, it does not gate symbols an application routes without one. A
symbol **with** a registered command is permitted only while its state is
[`actionable`](CommandState::is_actionable), so disabling or hiding one
command makes it inert on every surface at once: shortcut, menu, and
click alike.
