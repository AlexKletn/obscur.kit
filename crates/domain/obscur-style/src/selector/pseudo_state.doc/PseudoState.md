The set of interaction/state pseudo-classes currently active on a node.

This is an **input** to style, not an output. Per spec §2.5, focus (and,
more generally, interaction state such as hover, active, disabled, checked)
is *not* modeled as a CSS flag decided by the style engine. Instead the
interaction and focus subsystems produce these states, and the cascade
merely *reads* them when deciding whether a state pseudo-class matches.
Nothing in this crate decides when a node becomes hovered, focused, etc.;
it only consumes the bitset those subsystems supply.

Implemented as a `u32` bitset with no external dependency. Each associated
constant is a single-bit flag.
