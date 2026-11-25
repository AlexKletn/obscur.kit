Scrolls `node` into view inside its nearest scrollable ancestor, and
returns whether anything moved (spec §6.15 "Focused inputs may be scrolled
into view", §4.8 scroll-into-view).

The visible region is the container's scrollport **minus whatever the
on-screen keyboard occludes of it**. That subtraction is the whole point
on a device without a hardware keyboard: the field the user just tapped is
exactly the one the keyboard slides up over, and a scroll-into-view that
ignored the keyboard would decide the field was already visible and leave
it behind the keys.

A node already inside the visible region does not move — this is the
minimum scroll that reveals it, not a recentre.

# What it cannot do

Only a **scroll container** can scroll, so a field with no scrollable
ancestor cannot be revealed and this returns `false`. The root viewport is
not a scroller in this model, so a form laid out directly on the root has
nowhere to go — which is worth knowing before designing a screen for a
device whose keyboard covers half of it.
