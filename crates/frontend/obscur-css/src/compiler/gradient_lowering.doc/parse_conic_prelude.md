Parses the optional leading prelude of a `conic-gradient`: an optional
`from <angle>` followed by an optional `at <position>`. Returns [`None`] when
the part carries neither `from` nor `at` (so it is treated as the first color
stop) or when it is malformed (leftover tokens, bad angle/position).
