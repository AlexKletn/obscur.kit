Offers `command` to each link of `route` in order and stops at the first
with a handler bound for it. Returns whether a link claimed it.

Ungated and frame-free: the caller owns the [`CommandState`] gate, the
`prevent_default` check and the frame request. `route` comes from the
caller too, because building it needs the node path and this type has no
view of the tree.

The handler is called with `event`, so it can read §2.4's payload and
§2.3's `target` / `current_target` / trust. `current_target` is set per
link: the visited node for a [`CommandTarget::Node`], and `None` for a
[`Scope`](CommandTarget::Scope), [`Window`](CommandTarget::Window) or
[`App`](CommandTarget::App) link — those are levels of the routing chain,
not positions in the tree, so there is no node to name. The phase is
[`Target`](EventPhase::Target) for the first link (the origin node itself)
and [`Bubble`](EventPhase::Bubble) for every link beyond it, which is what
those links are: the event travelling outward.
