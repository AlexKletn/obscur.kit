Returns the nearest **strict** ancestor of `node` that is a *scoping*
component host (`NodeFlags::COMPONENT_HOST` without
`NodeFlags::OPEN_STYLE_SCOPE`), or [`None`] when nothing between `node` and
the root closes a scope over it.

The host itself is not its own ancestor, so a bare component host reports
[`None`] (it is authored content, not internal to a component). Its rendered
children — and their descendants — report the host: they are the scoped
output the boundary contains.

# An open host is transparent, not terminal

A component that declared [`StyleScope::Open`](obscur_view::component::StyleScope::Open)
(§4.15 "explicitly open/global scopes are possible") does not close a
boundary, so the walk **continues past it** rather than stopping and
reporting `None`. That distinction is the whole composition rule: an open
component used inside a scoped one is still inside the scoped one — opening
yourself lets the document reach you, it does not let you punch a hole
through somebody else's encapsulation.
