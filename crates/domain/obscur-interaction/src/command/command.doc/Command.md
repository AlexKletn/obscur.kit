A unified action descriptor (spec §6.7).

A `Command` is the single declarative descriptor of an action. The same
command drives menus, toolbars, the command palette, keyboard shortcuts, and
accessibility — a stable [`id`](Command::id) links every surface to one
action. This type carries only the static description (identity, title,
shortcut, icon); the dynamic runtime state (enabled/checked/visible) lives in
[`CommandState`](crate::command::CommandState), and dispatch lives in a
handler outside this pure-domain crate.

# Why the id is an `InternedSymbol`

The id is the **app-owned command symbol** every live surface already routes:
the symbol [`CommandRouter`](crate::command::CommandRouter) resolves a
shortcut to, the symbol a `MenuItem` fires, and the symbol the runtime
registers a handler for. It is deliberately *not* a string. This crate has no
string→symbol interner and cannot reach one — [`InternedSymbol::from_raw`] is
the type's only constructor, and the workspace's single interner belongs to
the CSS compiler (a `frontend` crate this `domain` crate may not depend on),
mints sequential ids that would collide with app-assigned symbols, and
documents itself as not shared. A string-keyed id could therefore never name
the thing that routes: it would be a parallel identity no surface could act
on. Keying on the routed symbol is what lets one `Command` drive every
surface.
