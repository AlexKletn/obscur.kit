The priority the winning declaration for each property had.

Kept because a *logical* property and its physical counterpart are two
separate `PropertyId`s that cascade independently, yet CSS says they
compete: `margin-left` and `margin-inline-start` on one element are decided
by ordinary cascade order. By the time the winner-per-property is picked,
that order is gone — so the priority is retained and the computed-value step
compares the two winners (spec §4.13 logical properties, §4.14 cascade).

Absent for a value inserted directly (e.g. by a test or a runtime override),
which is why every read of it is optional.
