Lowers a `background-size` value into a [`PropertyValue::BackgroundSize`]
(spec §5.2, CSS Backgrounds §3.9).

Accepts `cover`, `contain`, a single `auto`, or one/two
`[ <length-percentage> | auto ]` components (a single component leaves the
other axis `auto`). A component of `auto` (both axes) collapses to
[`BackgroundSize::Auto`]. Any other syntax yields [`None`] (the declaration
degrades to an unsupported-feature diagnostic and the initial `auto` applies).
