Lowers one non-`repeat` track: `auto`, `min-content`, `max-content`, an
`<fr>`, a `<length>`, or `minmax(min, max)`. Returns [`None`] for anything
else.

# An unrecognised track takes the whole declaration with it

Returning [`None`] here aborts the entire `grid-template-columns` value, not
just this track — the callers propagate it, and the declaration degrades to an
unsupported-feature diagnostic. That is deliberate (a track list with one
track silently dropped would shift every item into the wrong column, which is
worse than not laying out the template at all), and it is why the two content
keywords being missing here was worth more than it looked: a single
`min-content` in the list took the other tracks down with it.

`fit-content` is **not** here, and is still an unrecognised track: CSS Grid
spells it `fit-content(<length-percentage>)` — a function with an argument,
with no bare-keyword form on a track — and the argument is not modelled. It
therefore still degrades the declaration.
