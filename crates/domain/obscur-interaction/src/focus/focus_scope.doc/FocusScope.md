A focus-containment scope (spec §7.1).

A scope groups an ordered set of focusable members. Dialogs and menus open a
scope and set `trap` so that tab navigation wraps within the scope's members
instead of escaping to the surrounding UI — a *focus trap*. The scope itself
is a pure description; the runtime applies trapping using this data together
with the tab order and focus-restore stack.
