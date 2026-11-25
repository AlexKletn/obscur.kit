The nodes declaring a focus trap, outermost first.

An element declares one by carrying
[`FOCUS_TRAP`](obscur_interaction::focus::FOCUS_TRAP) with a value that is not
`Bool(false)` — the false case is a dialog stating it is *not* modal, which is
what lets a widget emit the property unconditionally and carry the author's
answer in it rather than in the property's presence.
