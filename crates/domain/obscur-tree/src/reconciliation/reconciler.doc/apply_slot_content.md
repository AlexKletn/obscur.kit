Refreshes a node's §2.12 slot content — the authored children a mounted
component projects into the outlets its own render declares — returning `true`
if it actually changed. A no-op (returning `false`) if `id` is stale.

The children are kept **only** when one of them carries a `slot="…"`
assignment. That gate is what makes retaining them affordable: an element's
children include its whole subtree, so keeping every element's would clone the
desired tree once per level of depth. A document that authors no slot stores
nothing and pays one `assigns_slots` scan of each child list.

This is deliberately *not* gated on [`hosts_component`]. Whether a node hosts
a component is only known after the plan this pass produces has been synced
against a registry, and the reconciler — a domain-tier engine — can see no
registry at all. Gating on the authored data instead keeps the decision where
the data is, and keeps the content available on the very first frame.
