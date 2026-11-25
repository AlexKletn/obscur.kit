Writes this frame's sampled `transform`/`opacity` into the computed styles, and
returns what it takes to undo them.

Group-compositing presentation values must reach paint and must never be visible
through `computed_style` (spec §4.17). Writing and then unwriting them costs work
proportional to the overlays; the copy this replaced cost work proportional to
the whole tree, on every frame of every animation.
