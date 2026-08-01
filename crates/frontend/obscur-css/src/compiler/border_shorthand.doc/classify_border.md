Classifies the components of a `border` / `border-<side>` value into its
`width` / `style` / `color` sub-values, in any order (spec CSS Backgrounds
and Borders §4).

Each component is a `<line-width>` (a length, or unitless `0`), a
`<line-style>` keyword ([`layout_keyword::border_style_from_name`]), or a
`<color>` (hex, `rgb()`/`rgba()`, or a named color). A component that fits
none of the three, or a second component of a category already seen, makes the
whole shorthand unparseable ([`None`]). An all-absent (empty) value is [`Some`]
with every field [`None`]; the caller reports that as unsupported.
