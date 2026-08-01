Maps command symbols to their [`Command`] descriptor and [`CommandState`]
(spec §6.7).

This is the join that makes one action descriptor drive many surfaces. Every
live surface routes an app-owned command [`InternedSymbol`] — a shortcut
resolved by [`CommandRouter`](crate::command::CommandRouter), a menu item
fired by a click, a handler registered on the runtime. The registry keys the
*description* (title, icon, shortcut) and the *state* (enabled, checked,
visible) on that **same** symbol, so a surface holding a symbol can ask for
the title to show, and any surface about to fire one can ask whether it is
actionable right now.

A symbol need not be registered: the registry describes commands, it does not
gate their existence. [`permits_dispatch`](Self::permits_dispatch) states the
resulting policy — an unregistered symbol is unconstrained, a registered one
is inert unless its state is actionable.
