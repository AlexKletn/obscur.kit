Rasterizes an **inset** `box-shadow` as an inner shadow confined to the box.

Unlike a drop shadow (a blurred rounded rect behind the box), an inset shadow
darkens the *inside* edges of the box. It is built in three steps, all clipped
to the box's rounded border box so nothing can bleed outside it:

1. Push a clip layer to the box's rounded rect (the confinement + isolation).
2. Fill that whole shape with the shadow color.
3. Punch a hole in the fill with a blurred rounded rect — the box deflated by
   `spread` and shifted by `offset` — composited with [`Compose::DestOut`], so
   only the soft frame between the box edge and the hole survives. The blur
   feathers the hole's edge, giving the standard inner-shadow gradient; with
   `blur == 0` and a positive `spread` the frame is a crisp inner ring.

The hole is punched with an opaque color because `DestOut` uses only the source
alpha; the visible shadow keeps the authored color's own alpha from step 2.

[`Compose::DestOut`]: peniko::Compose::DestOut
