Returns whether this range and `other` share at least one byte.

Ranges that merely touch at an endpoint (for example `0..2` and `2..4`)
do not intersect, and empty ranges never intersect.
