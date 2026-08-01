§4.7: "Hit testing accounts for:" / "- scrolling;" and
"- pointer-event policy;"

A 200x200 scroll container holds 20 items of 100px (item *i* at content
y `100i..100i+100`). At scroll offset 500, window point (50, 50) is content
y 550 — item 5. Overlaying a `pointer-events: none` box exactly over the
container must not change that: the overlay is excluded from hit testing
entirely rather than becoming a target whose event is later dropped.
