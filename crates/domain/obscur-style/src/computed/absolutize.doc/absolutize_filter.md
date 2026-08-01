Absolutizes the one filter op that carries lengths (`drop-shadow(...)`).

`blur(<length>)`'s radius is an `f32` by the time it reaches a computed value:
the CSS lowering resolved it with `Length::as_px`, so a `blur(1em)` is already
`0` before this pass sees it. That is a lowering-side limitation and is not
papered over here.
