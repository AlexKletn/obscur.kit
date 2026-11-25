Registers `handler` to run when `command` reaches `node` on §2.6's
routing chain — links 1 and 3 — replacing any handler previously
registered for that `(node, command)` pair.

This is what makes routing *positional*: the same command symbol can run
different logic depending on where focus sits. A handler here claims the
command and stops the chain, so nothing further up — no ancestor, not the
window, not the application fallback — sees it.

`node` must be a live [`NodeId`], which the app reads back from
[`tree`](Self::tree) after a frame has built it (the same way a node is
resolved from a stable id). A registration survives every frame that
keeps its node, and becomes unreachable if the node is removed.

Like [`on_command`](Self::on_command), `handler` may take nothing or take
the propagating [`UiEvent`] (see [`IntoCommandHandler`]).

[`UiEvent`]: obscur_interaction::event::UiEvent
