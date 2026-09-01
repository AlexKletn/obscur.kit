A hasher for integer-shaped keys, in place of the standard library's.

`HashMap`'s default is SipHash-1-3 under a per-process random seed. That pair
buys resistance to an attacker who chooses keys in order to collide them, and
it is the right default for a map holding anything that came in from outside.
It is the wrong default for a map keyed by a `NodeId`, a `PropertyId` or an
interned symbol — numbers this program handed itself — where the whole cost is
paid and none of the protection is needed.

The cost is not small. On a frame of a 227-node tree, SipHash accounted for
just over half of every instruction executed, most of it under
`ComputedStyle::get`, which the paint phase calls dozens of times per node.

The algorithm is `FxHash`, the one `rustc` uses on its own interned indices for
the same reason: rotate, xor the next word in, multiply by an odd constant.
It offers no collision resistance whatsoever against a chosen-key adversary,
so a map whose keys can be *authored* — a selector name, a font family, a CSS
custom property spelled by a stylesheet — should keep the standard hasher.
Use this one where the key is an identifier this crate minted.
