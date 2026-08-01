The desired presentation properties of a window (spec §6.2).

These are the caller-authored, mutable properties describing how a window
should appear: its title, size, position, presentation state, and a set of
boolean flags. They form the *intent*; a backend reconciles it against the
real platform window.

Blur/backdrop effects from §6.2 are intentionally omitted here: they are a
capability-gated effect owned by the backend port in a later sub-step, not a
plain property of the window model.

The multiple boolean flags mirror the independent presentation toggles
enumerated in spec §6.2 and are deliberately kept as flat fields rather than
folded into a bitflag type.
