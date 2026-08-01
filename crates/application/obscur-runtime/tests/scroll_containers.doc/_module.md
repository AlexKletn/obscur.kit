A per-element scroll container, and the one question that makes it hard:
*where does a node have to be for the user to see it?* (CSS Overflow 3,
spec §6.15, §4.8.)

Eleven tests over one 200×120 scroller of six 40 px rows — 240 px of content in
a 120 px box, so the scrollable range is exactly 120 and every expected offset
below is arithmetic a reader can check.

Four of them are the container itself: overflow makes it scrollable, `scroll_by`
and `set_scroll_top` clamp to that range, a scrolled container wraps its content
in a `translate(0, −top)` that is *elided* at zero, and the pointer hit-test
composes the same offset so a press lands on the row the user sees rather than
the row that was painted there before. The last of the four is the guard for
everything the feature must not have cost: a view with no scroll container at
all composes a byte-identical display list.

The other seven are `scroll_into_view`, which is a harder claim than it looks —
it scrolls the **minimum**, in either direction, and answers `false` rather than
lying when there is nowhere to go. Three of them bring the keyboard in, which is
why §6.15 exists: the region a field is revealed *into* is the scrollport minus
the keyboard insets, so the same row has to travel further, a keyboard that
rises over an already-focused field triggers the reveal on its own, and a field
flush against the bottom of the content cannot be lifted at all. That last one
is a limit, not a bug — and it is written down as a test so it stays a known
one.
