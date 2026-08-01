Returns a copy with the [`highlight`](Self::highlight) clause range set.

The range is interpreted as byte offsets within [`text`](Self::text); the
caller is responsible for keeping it within bounds and on char
boundaries.
