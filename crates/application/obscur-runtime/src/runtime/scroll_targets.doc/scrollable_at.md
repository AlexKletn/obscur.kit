The innermost **user-scrollable** container under a content-space `point`,
or [`None`] when the point is over nothing the user may scroll.

Only what the user may scroll. A `hidden` box is a scroll container in the
CSS sense — it clips and can be scrolled programmatically — but the wheel
must pass through it, or a row that clips a long name eats the scroll meant
for the list it sits in.
