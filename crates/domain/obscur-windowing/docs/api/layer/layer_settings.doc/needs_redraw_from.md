Whether moving from those settings to these changes what must be drawn.

**Moving a surface is not repainting it.** A margin, a depth band, a
reservation, a keyboard policy, a pointer policy — all of them change where the
surface is or how the desktop treats it, and none changes a pixel of its
content. A drag is made of margin changes, so answering each one with a frame
re-renders identical content once per position: sixty whole frames a gesture,
none of which draws anything new.

What does need a frame is the **extent**, and the anchor is part of that:
anchoring both edges of an axis stretches the surface, so the extent follows
from the anchor as much as from a stated width or height. A different display
counts too — it may bring a different scale, which is the same question one
level up.

A host still gets a frame for a real resize from the platform itself, which
sends a configure whenever the geometry it chose has changed. This answers the
narrower question of whether to ask for one *in advance*.
