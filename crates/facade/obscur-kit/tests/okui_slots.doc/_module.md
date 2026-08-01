End-to-end: a §2.12 slot contract is **enforced**, on both front ends
(spec §2.12, §2.13).

Slots existed only as types. `SlotContract`/`SlotCardinality` had no
production consumer, nothing parsed a `slot="…"`, and no document could break
a rule that nothing checked. This file drives the whole chain that now exists:
a component declares its slots with `#[derive(Contract)]`, the declaration
crosses into the compiler's string contract, and a document assigning children
to those slots is checked against it — through the **real** `Compiler` for the
`.okui` file path, and through the **real** `okui!` macro for the inline one.

# What is checked here, and what is checked elsewhere

What is checked *here* is the **assignment**: a `slot="…"` must name a
declared slot, satisfy that slot's own child rule, and the slot's cardinality
must hold. All of it is a compile-time verdict, reached before a runtime
exists.

**Projection** — the content actually reaching the outlet — is real, and is
tested in `okui_slot_projection.rs` rather than here, because it needs a
runtime and a registered component. The two halves meet at the reserved
`slot` symbol these front ends lower to: this file pins that the assignment
reaches the tree (`a_slot_assignment_is_the_reserved_property_the_tree_carries`),
and that file pins that the runtime routes on it.
