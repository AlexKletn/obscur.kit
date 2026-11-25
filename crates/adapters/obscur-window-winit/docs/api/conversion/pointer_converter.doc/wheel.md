winit's wheel report to a [`WheelEvent`], keeping both axes and the unit.

A pure function over winit values, so it is unit tested directly rather than
through an event loop — the same shape as the rest of this type.

Two things it deliberately does NOT do. It does not multiply lines into pixels:
how far a notch scrolls is the toolkit's policy and lives on `UiRuntime`, and
flattening here is what used to leave the runtime unable to tell a notched
wheel from a trackpad. And it does not drop the horizontal delta, which a `_`
pattern here used to do — the runtime scrolls vertically only, but an
application with a horizontal list needs the number the platform reported.
