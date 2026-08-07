Resolves the `content` (and its `color`) of `element`'s generated box named
`pseudo` (`before` / `after`) from `candidates` — the rules that could target
it — or [`None`] when no matched rule sets a `content` string.

An empty `candidates` is the common case and answers [`None`] without matching
anything: no rule in the sheet set names this pseudo-element, so no element can
match one.
