Lowers a function value: `var(--name)` to a [`PropertyValue::CustomRef`], any
CSS Color 4 `<color>` function (`rgb()`/`hsl()`/`oklab()`/`oklch()`/`lab()`/
`lch()`) to a colour, and `calc()`/`min()`/`max()`/`clamp()` (and a
bare parenthesised group) to a [`PropertyValue::Calc`] symbolic length
expression (spec §4.13). Any other function is not modelled and yields
[`None`].
