The custom properties one node declares, cascaded from the sheets that
matched it.

The companion to [`resolved_for_node`], and separate from it because custom
properties are keyed by name rather than by property: they cannot travel in
the same `ResolvedDeclarations`, and §4.14 resolves them a step earlier than
everything else so that a `var()` has an answer before any computed value is
produced.

A component's internal element declares none: the scoping that keeps an
application sheet's longhands out of a component keeps its custom properties
out too. Inheritance carries the host's down regardless, which is the door
that stays open on purpose — it is how a component is themed from outside
without its boundary being crossed.
