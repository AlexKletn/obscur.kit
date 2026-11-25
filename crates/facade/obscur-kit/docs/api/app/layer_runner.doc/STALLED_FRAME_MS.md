How long to wait for a frame callback that is not coming before drawing anyway.

The gate on building frames is the compositor`s callback, and a gate with no way
out is a surface that never repaints: a compositor may stop calling back for as
long as it likes — the surface is hidden, animating from a snapshot, on another
workspace — and it owes nobody an explanation for it.

So the gate expires. Generous, because it is a failsafe and not a pacing device:
reaching it means a frame is being built that will probably not be shown, which
is exactly the cost this whole mechanism exists to avoid, and it should happen
when something is wrong rather than routinely.

Before the first callback ever arrives the loop falls back to the animation step
instead, so a compositor that does not implement frame callbacks at all is paced
exactly as it was before any of this existed.
