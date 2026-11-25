Returns the inverse unit: each edit inverted *and* the sequence reversed.

Edits applied in order `e0, e1, …, eN` must be undone in the opposite
order — the inverse of `eN` first — because each edit's byte offsets
assume the prior edits are already in place. The reversed, inverted
sequence therefore restores the pre-unit state. The kind is preserved.
