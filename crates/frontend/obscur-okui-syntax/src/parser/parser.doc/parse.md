Parses `source` into a lossless [`ParseResult`].

The returned tree always round-trips the input
(`result.tree().text() == source`); any problems are reported in
`result.errors()`.
