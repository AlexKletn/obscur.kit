The names on each grid line, indexed by 0-based line position (line `i` is
the `i`-th boundary, so line `0` is before the first track). Empty when no
`[name]` group was authored; otherwise `tracks.len() + 1` buckets, each a
(possibly empty) list of the names on that line.
