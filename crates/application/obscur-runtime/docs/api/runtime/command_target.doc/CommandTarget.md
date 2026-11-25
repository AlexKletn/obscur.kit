One link of the §2.6 command routing chain.

The spec's chain is *"focused node; focus scope; ancestors; window;
application fallback"*. A `CommandTarget` names one of those levels for a
concrete dispatch, and
[`UiRuntime::command_route`](super::UiRuntime::command_route) returns them in
the spec's order. A dispatch offers the command to each link in turn and
stops at the first that has a handler registered for it — the first link to
*claim* the command handles it, and no later link sees it.

[`Node`](Self::Node) covers **two** of the spec's five levels: the route's
first `Node` is the focused node (level 1), and every `Node` after the
[`Scope`](Self::Scope) links is an ancestor (level 3). They are the same kind
of link — a handler registered at a node — so they share a variant; their
*order* in the route is what distinguishes them.
