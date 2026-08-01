The [`ComponentId`] a **user** component tag binds to: FNV-1a 64 of the name,
lifted into a high namespace (top bit set).

This is the id `#[derive(Contract)]` puts on `ComponentContract::id` for the
same name, and the whole reason an author's `<Card/>` and a host's
`register_component(…)` meet without either consulting the other: the name is
the shared secret. The derive does not re-implement it — it calls
[`component_id_for`] — so the two cannot drift.

The top bit is set for the same reason [`fallback_symbol`] sets it: every
built-in `COMPONENT_ID` is a small low-range constant, so a hashed name can
never *alias* one. That makes the two namespaces disjoint by construction
rather than by the odds on a 64-bit hash.
