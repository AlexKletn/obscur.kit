Reserved [`ComponentId`] of a **virtual-collection marker** — the node a
`<VirtualFor>` lowers to, standing in for the window of rows that
realization materializes (spec §2.11, §4.9).

Nothing is ever registered under this id and no node ever retains it:
[`realize_virtual_collections`] replaces every marker with a viewport-bounded
window of its template before the output reaches the reconciler.

It sits alongside [`SLOT_OUTLET`](super::SLOT_OUTLET) in the reserved
`0x00B0_xxxx` range — below the primitive (`0x00C0_xxxx`) and widget
(`0x00D0_xxxx`) ranges and clear of the top-bit namespace a user component
hashes into — so it can alias neither a shipped component nor an author's.
