Applies the CSS Backgrounds 3 §4.5 overlapping-curves correction for `rect`.

The spec: "Let f = min(Li/Si), where i ∈ {top, right, bottom, left}, Si is
the sum of the two corresponding radii of the corners on side i, and Ltop =
Lbottom = the width of the box, and Lleft = Lright = the height of the box.
If f < 1, then all corner radii are reduced by multiplying them by f."

The *corresponding* radius of a corner on a side is the one running **along**
that side: the horizontal (`x`) radii sum on the top and bottom, the vertical
(`y`) radii on the left and right.

```text
f = min( W / (tl.x + tr.x),   // top side
         W / (bl.x + br.x),   // bottom side
         H / (tl.y + bl.y),   // left side
         H / (tr.y + br.y) )  // right side
```

A side whose radius-sum is `0` is skipped — treated as `+∞` so it never
constrains, avoiding a divide-by-zero. If `f < 1` **both** radii of every
corner are multiplied by `f`, which is what the spec's note means by
"quarter circles remain quarter circles and large radii remain larger than
smaller ones"; if `f ≥ 1` the radii already fit and are returned unchanged
(byte-identical). This is the single scale applied to all four corners,
unlike an independent per-corner `min(w,h)/2` clamp which distorts
asymmetric radii on non-square boxes.

`border-radius: 50%` is the fixed point this protects: on any box the four
side sums equal exactly that side's length, so `f == 1` and the authored
ellipse survives untouched.
