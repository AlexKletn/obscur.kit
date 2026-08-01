What a [`PerformanceSpan`](crate::performance::PerformanceSpan) is attributed
to, beyond its subsystem (spec §7.9).

§7.9 lists frame, window, subsystem, node, layout context, resource,
invalidation cause and plugin contribution. Subsystem lives on the span
itself; the rest that this crate can *name* live here.

# Every field is optional, and that is the honest shape

A span is attributed to what its emitter actually knows. A frame-phase span
knows its frame and nothing narrower — there is no single node a whole layout
phase is "about". A per-node span would set `node`; a resource load would set
`resource`. Forcing a value where the emitter has none would mean inventing
attribution, which is worse than reporting none: a consumer cannot tell an
invented answer from a measured one.

# What is deliberately absent

**Layout context**, **invalidation cause** and **plugin contribution** from
§7.9's list are not here. Each is named by a type that lives *above* this
foundation crate (`InvalidationReason` is a runtime concept, a layout context
a layout one), so carrying them would mean either a dependency inversion or a
duplicated enum that silently drifts from the real one. They wait for their
owners to expose an identity this tier can hold.
