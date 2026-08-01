Lowers one `blur(<length>?)` argument list into a non-negative blur radius in
logical pixels.

The `blur()` grammar takes a single optional non-negative `<length>`; `blur()`
with no argument is the CSS initial `0`. The radius resolves to absolute `px`
(a unitless `0` allowed); a relative/viewport unit that cannot resolve at lower
time falls back to `0` — a documented approximation matching the px-only
transform-translate lowering. Returns [`None`] for more than one argument, a
non-length token, or a **negative** radius (invalid per spec).
