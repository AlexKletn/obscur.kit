Reserved name of the **default slot** — the one an outlet declares by naming
no slot at all, and the one content reaches by carrying no `slot="…"`
(spec §2.12).

A [`SlotContract`] names its slot with a symbol, and the default slot has no
authored name to intern, so this is the symbol a contract declares it under.
It names nothing in a document: an author never writes it, because the way to
reach the default slot is to write no assignment. It sits beside [`SLOT`] in
the `0x0000_03xx` slot namespace.

[`SlotContract`]: crate::contract::SlotContract
