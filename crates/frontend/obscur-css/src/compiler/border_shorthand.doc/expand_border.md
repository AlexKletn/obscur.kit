Expands a `border` / `border-<side>` shorthand into the present
`width` / `style` / `color` longhands for each targeted edge (spec CSS
Backgrounds and Borders §4).

The value is classified once into its sub-values ([`classify_border`]); each
authored sub-value is then emitted for every edge in `sides` (all four for
`border`, one for `border-top` / …). Omitted sub-values are **not** emitted —
a documented simplification of CSS's reset-to-initial semantics — so
`border: 2px solid red` sets width + style + color on every side while
`border-top: 1px solid red` sets them on the top edge only. An unparseable or
empty value degrades the whole shorthand to a single
[`CompiledDeclaration::Unsupported`].
