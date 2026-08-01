`font-size` accepts a math function, so type can follow the window
(css-values-4 §10, css-fonts-4 §3.5).

It did not. A `calc()` / `min()` / `max()` / `clamp()` value is a
`PropertyValue::Calc`, not a `Length`, and the font-size path read only the
length — so the whole declaration was dropped and the element kept the
initial `medium`. Every math function computed to **16px at every window
size**, which quietly ruled out responsive typography altogether:
`clamp(16px, 5vmin, 30px)` is exactly how type is meant to track the window
between bounds.

The tell was that it looked like *nothing*: a fixed label in a key that grows
with the window drifts out of proportion, and reads as a centring bug rather
than as a dropped declaration.
