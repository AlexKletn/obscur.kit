Recognizes a [`LongPress`](GestureKind::LongPress): a contact that stays put
long enough to mean something other than a tap (spec §6.16).

A touchscreen has one button and no modifiers, so everything a second button
or a modifier would reach on a desktop has to be reached by holding still.
That makes this the only route to context actions on a tablet, and it is the
one gesture that fires **while nothing is happening** — no event arrives from
a finger that rests, so recognition cannot be driven by input alone. The
consumer advances it with elapsed time instead, which is why
[`advance`](Self::advance) exists and why this type reads no clock: it is
told how much time passed, like everything else in this crate.

# Lifecycle

`Possible` → `Recognized` (held past the threshold, still within the slop) or
`Failed` (the contact moved too far, or lifted first). Recognition happens
**once**: [`advance`](Self::advance) reports `true` on the step that crosses
the threshold and never again, so a consumer that keeps ticking does not open
a context menu on every frame the finger stays down.
