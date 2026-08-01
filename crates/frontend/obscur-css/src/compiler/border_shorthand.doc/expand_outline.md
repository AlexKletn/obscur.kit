Expands an `outline` shorthand into the present `outline-width` /
`outline-style` / `outline-color` longhands (spec CSS UI §outline).

The value is classified with the SAME `<line-width>` / `<line-style>` /
`<color>` grammar as `border` ([`classify_border`]), in any order; each
authored sub-value is emitted for its outline longhand. Like `border`, omitted
sub-values are not emitted (a documented simplification of reset-to-initial),
so `outline: 2px dashed #333` sets all three while `outline: 2px` sets only the
width. `outline-offset` is a separate longhand (not part of this shorthand). An
unparseable or empty value degrades to a single
[`CompiledDeclaration::Unsupported`].
