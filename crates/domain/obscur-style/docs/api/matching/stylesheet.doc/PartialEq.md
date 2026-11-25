Content equality: same origin, same rules, in the same order.

Hand-written rather than derived so that [`stamp`](Stylesheet::stamp) — an
identity, not content — stays out of it. Deriving it would make every sheet
unequal to every other sheet built separately, which is the opposite of what
every existing caller of `==` means.
