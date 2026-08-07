Whether `node`'s computed `pointer-events` is `none`, which makes it
transparent to the pointer (§4.7 "pointer-event policy").

Only the node's own computed value is read, and that is enough for the whole
subtree rule: `pointer-events` inherits, so every descendant of a `none`
element has already computed `none` too — and a descendant that declared
`auto` has computed `auto` and stays hittable, which is the behaviour the
overlay-with-a-live-button pattern depends on.

An absent value means the property was never declared anywhere up the chain,
i.e. the initial `auto`: hittable.
