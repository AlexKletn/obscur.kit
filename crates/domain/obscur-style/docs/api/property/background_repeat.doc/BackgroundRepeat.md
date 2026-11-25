The computed value of `background-repeat`: whether the sized image tile is
repeated (tiled) along each axis to fill the painted box.

The scene coordinator turns this into a pair of per-axis repeat flags
(`repeat_x` / `repeat_y`) at paint time:

* [`BackgroundRepeat::Repeat`] — tile on both axes (the CSS initial).
* [`BackgroundRepeat::NoRepeat`] — a single tile, no tiling.
* [`BackgroundRepeat::RepeatX`] — tile horizontally only.
* [`BackgroundRepeat::RepeatY`] — tile vertically only.

The CSS `space` and `round` keywords are **folded to** [`BackgroundRepeat::Repeat`]
(a documented approximation): their gap-distribution / integer-fit behaviour is
not modelled, so they tile like plain `repeat`.
