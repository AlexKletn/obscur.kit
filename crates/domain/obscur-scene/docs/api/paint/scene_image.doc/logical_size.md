The image's intrinsic size in **logical** CSS pixels — its pixel
dimensions divided by its [`scale`](Self::scale) (spec §5.16).

This, not the raw pixel count, is the intrinsic size CSS sizing and
`object-fit` are resolved against: a 200×200 `@2x` asset stands in for a
100×100 logical box and must lay out at that size, so that the device-scale
root transform then maps it back to 200×200 physical pixels — a crisp 1:1
mapping. At the base density this is exactly the pixel size.
