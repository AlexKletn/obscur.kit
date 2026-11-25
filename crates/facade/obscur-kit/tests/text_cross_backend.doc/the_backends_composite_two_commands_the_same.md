Two text commands composite to the same alpha on both backends, in real pixels.

The raster form of the guard: both backends must reach 192/255 at the probe, and
must not disagree about coverage anywhere in the raster. Where its GPU-free
sibling reads the Vello side out of the encoding, this one reads it off the
device, through the same `render_to_image` the present path uses.

Watched failing: with the reverted fold temporarily restored in
`SceneConverter`, this read `cpu 192, vello 128` and failed on the first
assertion.

Needs an adapter, so it is `#[ignore]`d — counted as ignored, never as a pass —
and fails outright if it is asked for where no adapter exists.
