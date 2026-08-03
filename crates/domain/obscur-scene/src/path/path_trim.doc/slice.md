The portion of the path between two fractions of its length, as lines.

Out of range clamps — `0.0..1.0` is the whole path, and asking for more of it is
not an error. Out of ORDER is empty rather than reversed: a caller asking for
`0.8..0.2` has a bug, and quietly drawing the complement would hide it.
