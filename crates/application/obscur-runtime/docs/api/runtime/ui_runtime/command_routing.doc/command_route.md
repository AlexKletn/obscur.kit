Returns §2.6's command routing chain for a command originating at
`origin`, in the spec's order: **focused node; focus scope; ancestors;
window; application fallback**.

Concretely, for `origin = Some(node)`:

1. [`CommandTarget::Node`]`(origin)` — the focused node itself;
2. one [`CommandTarget::Scope`] per open [`FocusScope`] containing
   `origin`, **innermost first** (a nested dialog's scope precedes the
   dialog's own);
3. one [`CommandTarget::Node`] per ancestor of `origin`, nearest first,
   up to the root;
4. [`CommandTarget::Window`] — this runtime's own handlers;
5. [`CommandTarget::App`] — the shared [`AppCommands`] fallback.

With `origin = None` the positional links have no anchor, so the route is
just window then application. The two non-positional links are *always*
present, so a route is never empty.

Note that the scope links precede the ancestors even though a scope's
members sit *inside* those ancestors: that inversion is §2.6's own
ordering, not an accident of this implementation.
