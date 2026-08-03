The node Tab moves to from `current`, or `None` when the ring is empty.

A `current` that is not in the ring — which a pointer press produces routinely —
starts from the first stop going forward and the last going backward, which is
`TabOrder`'s own answer and the reason it distinguishes that case.
