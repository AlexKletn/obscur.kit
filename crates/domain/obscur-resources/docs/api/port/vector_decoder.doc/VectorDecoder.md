Decodes source bytes into a [`SceneVector`] (spec §5.17).

The vector counterpart of [`ResourceDecoder`](crate::port::ResourceDecoder),
and a separate port for a reason: a raster decoder's product is a buffer of
pixels, while this one's is *geometry plus deferred paint*. Forcing both
through one port would mean erasing the vector document back into bytes —
which is exactly what §5.17 ("SVG remains vector through the scene pipeline")
forbids.

A host installs an implementation; the runtime uses it when an element's
source names a vector document. With none installed, vector sources simply do
not paint — the same policy an absent raster decoder has.
