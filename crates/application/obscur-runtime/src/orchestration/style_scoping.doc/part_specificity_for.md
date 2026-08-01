Returns the specificity of the highest-ranked `::part(want)` selector in
`selector`'s owning list that both targets `want` and whose host constraints
match `host`, or [`None`] when none do.

The caller has already decided the subject element *exports* `want`; this
only decides whether the `Component` half of `Component::part(want)` matches
the host.
