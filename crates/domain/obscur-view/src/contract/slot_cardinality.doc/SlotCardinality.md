The number of children a named slot may accept (spec §2.11, §2.12).

Cardinality is an independent trait of a slot, so it lives in its own type
rather than being encoded as loose counts on the slot contract. This keeps
the common shapes explicit and typed.
