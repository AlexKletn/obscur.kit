§4.7: "Hit testing accounts for:" / "- scrolling;" and
"- pointer-event policy;"

A 200x200 scroll container holds 20 items of 100px (item *i* at content
y `100i..100i+100`). At scroll offset 500, window point (50, 50) is content
y 550 — item 5. Overlaying a `pointer-events: none` box exactly over the
container must not change that: the overlay is excluded from hit testing
entirely rather than becoming a target whose event is later dropped.

The scrolled probe alone does not prove the policy. At offset 500 the folded
point is (50, 550), outside the overlay's own 200x200 box, so the overlay
would be missed even by a hit test that had never heard of `pointer-events`.
Two unscrolled probes carry the weight instead. There the folded point is
(50, 50), squarely inside the overlay, and the overlay carries `z-index: 1`
so it outranks the deeper item beneath it:

* `.over-none` (`pointer-events: none`) must let the click reach item 0, and
* `.blocker` — the identical box without the declaration — must swallow it,

which is what makes the first outcome the property's doing rather than the
geometry's.
