Lowers a `background-repeat` value into a [`PropertyValue::BackgroundRepeat`]
(spec §5.2, CSS Backgrounds §3.4).

Accepts the single keywords `repeat` / `no-repeat` / `repeat-x` / `repeat-y`,
the folded `space` / `round` (→ `repeat`, a documented approximation), or the
two-value `<repeat-style>{2}` axis form (each `repeat`/`space`/`round` = tile,
`no-repeat` = no tile). Any other syntax yields [`None`].
