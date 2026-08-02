Refreshes a node's §2.12 slot content — the authored children a mounted
component projects into the outlets its own render declares — returning `true`
if it actually changed. A no-op (returning `false`) if `id` is stale.

The children are kept when one of them carries a `slot="…"` assignment, **or**
when `component` is an id a component can be mounted under
(`may_project_content`). Two gates, because there are two ways content reaches
an outlet and only one of them is visible in the children: content bound for
the *default* slot names no slot, and `<button>Save</button>`'s only child is a
text run, which can carry no assignment at all. Reading the host's id instead
is what keeps that content.

Between them the gates are what makes retaining affordable: an element's
children include its whole subtree, so keeping every element's would clone the
desired tree once per level of depth. A `div` is a primitive, below the
mountable range, so an ordinary document stores nothing and pays one
`assigns_slots` scan per child list; what stores is a widget or user-component
node, whose content is its own control's, and which is rare in any document.
An id that mounts nothing (`<table>`, today) pays the clone and nothing reads
it.

This is deliberately *not* gated on [`hosts_component`]. Whether a node hosts
a component is only known after the plan this pass produces has been synced
against a registry, and the reconciler — a domain-tier engine — can see no
registry at all. Gating on the authored data instead keeps the decision where
the data is, and keeps the content available on the very first frame.
