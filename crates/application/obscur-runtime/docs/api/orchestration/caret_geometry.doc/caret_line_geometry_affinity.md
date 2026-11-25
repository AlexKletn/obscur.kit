Resolves a caret byte `index` to its painted geometry like
[`caret_line_geometry`] but honoring the caret's
wrap-boundary [`CaretAffinity`] (spec §5.11).

A byte at a **soft**-wrap boundary is shared by two visual positions — the
end of the earlier line and the start of the later one. The default
[`Downstream`](CaretAffinity::Downstream) binds it to the later line's
leading edge (exactly what `caret_line_geometry` does), so a default-affinity
caret — and every caret NOT on a soft boundary — is byte-identical.
[`Upstream`](CaretAffinity::Upstream) instead binds it to the earlier line's
trailing edge: its box top/height, its alignment `x_offset`, and its full
used content width (every glyph on that line has a cluster before `index`).
A **forced** break (`\n`, spec §4.1) is NOT a soft boundary — its two sides
are different bytes — so affinity is inert there (`content` distinguishes the
two cases).
