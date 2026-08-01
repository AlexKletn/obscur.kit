The three interaction fields the state pass examined and refused to extract —
`hover`, `focused` and `modifiers` — whose entire public surface is one reader
each.

They are together because the refusal is what they have in common: each is read
across the whole runtime and none shares an invariant with the others, so no
entity could own them, and the docs/runtime-decomposition.md ledger records why.
What is left of them on the public API is three questions about the last input
that was dispatched.

Their one writer is here for the same reason. `recompute_pseudo_states` turns
`hover`, `focused` and the outstanding presses into the `:hover`, `:focus` and
`:active` a selector matches against, and it reads them together — so it belongs
with the fields it projects rather than with the pointer dispatch that is
currently its only caller. What calls it is an event; what it is *about* is
these fields.
