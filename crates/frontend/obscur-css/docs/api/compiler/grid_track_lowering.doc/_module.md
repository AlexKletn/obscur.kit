Track-list lowering: `grid-template-columns` / `-rows`, `grid-auto-columns` /
`-rows` and `grid-template-areas` (css-grid-2 §7.2, §7.3, §7.5).

A track list is a recursive grammar of its own — `minmax()`, `repeat()` with
its `auto-fill` / `auto-fit` forms, the content-sizing keywords and the
interleaved `[line-name]` groups — and it is the only place in the crate
where a value expands into a *list whose length is computed*. That recursion,
and the [`MAX_REPEAT`] bound that stops an author's `repeat(1e9, …)` from
allocating, are what make it an entity rather than one more property arm.
