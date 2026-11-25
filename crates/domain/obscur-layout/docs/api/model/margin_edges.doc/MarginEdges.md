The four margin edges of a box (spec §4.2).

The physical twin of [`Insets`], differing in exactly one way: each edge is a
[`Margin`], so it can be `auto`. That is the whole reason this type exists —
`Insets` is four `f32`s, and an `auto` margin has no `f32` to be until the box
has been measured and the leftover space is known.

Border and padding stay [`Insets`]: neither accepts `auto` in CSS, so neither
has anything to defer.
