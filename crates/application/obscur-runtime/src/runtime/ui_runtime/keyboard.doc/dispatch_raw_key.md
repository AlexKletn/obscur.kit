Delivers `keydown` / `keyup` to the focused node's binding, and reports whether a
listener refused the default action.

Free for an application that binds neither: with no `on:keydown` in the focused
node's ancestor chain there is no command to dispatch, and this returns `false`
after one ancestor walk.

The target is the focused node, which is also why an unfocused runtime delivers
no raw key event: there is no node for it to target. A shortcut still resolves in
that state — §2.6's route ends at the window surface and the application — and
that is deliberately a different question from this one.
