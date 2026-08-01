Mutates the value in place through `f` and advances the generation.

The generation advances unconditionally, matching the spec's "mutation
bumps it" rule: the toolkit does not diff values to decide whether a
change occurred.
