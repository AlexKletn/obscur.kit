Every authored `<color>` syntax, lowered to one [`StyleColor`] (CSS Color 4).

The named-colour table, the hex forms, the legacy comma and modern
space-separated `rgb()` / `hsl()` argument shapes, and the four
perceptual spaces (`oklab()`, `oklch()`, `lab()`, `lch()`) all answer the
same question, and every one of them is reachable from any property that
takes a colour: a longhand, a border component, a shadow, a filter operand, a
gradient stop, the `background` shorthand. That fan-in is the seam — the
colour grammar belongs to no single property, so it belongs to none of their
files.
