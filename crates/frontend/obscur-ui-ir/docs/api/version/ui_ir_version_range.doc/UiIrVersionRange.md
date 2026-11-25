An inclusive range of [`UiIrVersion`]s a component supports (spec §2.5).

Migrations and readers advertise the span of IR versions they accept as a
`[min, max]` inclusive range. The range invariant is `min <= max`; a
constructor given an inverted `min > max` clamps `max` up to `min`, yielding
the single-version range `[min, min]` rather than an empty or reversed one.
