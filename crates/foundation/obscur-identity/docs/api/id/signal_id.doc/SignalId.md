Stable, opaque identifier for an observable state source (spec §3.1).

State enters the UI through typed props, local component state, and
**read-only signal/observable sources**. A `SignalId` names one such source so
a component can declare that its render depends on it, and the runtime can
invalidate exactly the components that depend on a source that changed —
rather than everything.

Backed by an integer handle; the numeric value carries no meaning outside the
subsystem that mints it and must never be treated as a public schema. An
application typically declares one `const` per source.
