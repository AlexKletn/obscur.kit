The authored stylesheet, keyed to the classes/attributes the `.okui` declares.
`#app` is an id selector, `[type=card]` is an **attribute** selector (matching
every element the markup wrote `type="card"` on), and the rest are class
selectors — all three selector kinds are driven by attributes the `.okui`
bridge carries through.

It used to read `card { … }`, a *type* selector, because the tag and the
authored `type="…"` shared one reserved symbol and the author's value won it:
a `<div type="card">` reached the tree with CSS type `card` and none of `div`.
They are two symbols now — the tag says what a thing is, `type` says which
kind — so the rule says which one it means.
