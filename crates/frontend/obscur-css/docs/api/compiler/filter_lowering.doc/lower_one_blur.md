Lowers one `blur(<length>?)` argument list into a non-negative radius
expression.

The `blur()` grammar takes a single optional non-negative `<length>`; `blur()`
with no argument is the CSS initial `0`. An absolute radius comes back as a
pixel leaf; a font- or viewport-relative one (`1rem`, `2vw`, `calc(1em + 2px)`)
comes back UNRESOLVED, for computed value to fold — where it used to be run
through `Length::as_px`, which answers [`None`] for every relative unit and was
then read as a radius of `0`, so a `blur(1rem)` backdrop rendered sharp.

Returns [`None`] for more than one argument, a non-length token, a **negative**
radius, or a **percentage** (both invalid per spec — `blur()` has no percentage
basis).
