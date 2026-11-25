The attachment load op the blit's render pass runs under, named so that the
choice has one place, one measurement, and one test that notices it moving.

## The two ops are the same picture

`Load` and `Clear` produce identical pixels here. The triangle covers the whole
attachment, the pipeline blends nothing, and the fragment shader samples only
`source` — so whatever `Load` loaded is overwritten without ever being read.
That is not a reading of the shader, it is asserted texel-by-texel over a full
readback in
`the_two_load_ops_draw_the_same_pixels_and_here_is_what_each_costs`.

So the choice is a cost question with no correctness in it, and cost is the
only thing allowed to decide it.

## What it costs here, measured

`Clear` was carried in an earlier pass on an argument about tiled GPUs, against
the only measurement anyone had, which said `Clear` was worse. That is backwards
and this is the line going back.

Measured again, from scratch, on this desktop's Intel Arc MTL iGPU through the
REAL pipeline of this file (the test lives in this module and reaches
`PremultiplyBlitter`'s own pipeline, sampler and bind group; only the pass's
load op differs between the arms), at the size the op's cost is proportional to
— 2944x1004, the tablet panel in physical pixels — fifty passes to a submit,
arms alternating, us per pass:

```text
        run 1               run 2               run 3               run 4
Load    424 / 362 / 371     410 / 402 / 380     400 / 390 / 413     394 / 392 / 382
Clear   410 / 422 / 416     456 / 447 / 434     438 / 440 / 472     433 / 422 / 401
diff    +45.2               +44.7               +39.7               +30.0
```

`Clear` is slower in four runs out of four, and by more than the earlier pass
reported (22-28 us). An immediate-mode GPU has no tile to prime: the clear is
work that the triangle then overwrites, and the load it replaces was free.

## The hypothesis this does NOT settle, and the number that would

On a tiled renderer — Adreno, which is what the tablet has — `Load` is supposed
to mean "fetch this attachment from system memory into tile memory on entry to
every tile", about 11.8 MB of reads a present for that panel, over a bus the CPU
shares, to supply texels the pass discards. `Clear` would initialise the tile in
registers instead and skip all of it.

That may well be true. Nothing here can weigh it: the tablet is not available to
this repository, and no software rasteriser on this machine reproduces a tiler's
attachment traffic. So the hypothesis is written down rather than acted on, and
it is written down in a falsifiable form:

> Set `LOAD_OP` to `wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)`, build for
> the tablet, and read `[layer-loop] present` p50 across several runs against
> the same build with `Load`.
>
> - A drop of **0.5 ms or more** in `present` p50, reproduced across runs, is
>   the attachment read being skipped: 11.8 MB at that panel's plausible
>   effective bandwidth is 0.5-1.2 ms, so a saving of that size is the predicted
>   effect and nothing else on that path is that big. `Clear` then belongs here,
>   and this file should say so with those numbers in it.
> - A drop **under 0.2 ms**, or none, refutes it. `Load` is then right on both
>   machines and this note can be deleted rather than kept as a maybe.
> - Between the two, the run is not conclusive and more runs are the answer, not
>   a decision.

Until one of those numbers exists, the op is the one that measured faster on the
only hardware that can be measured. A choice made against its own measurement
because a second machine might disagree is the thing this pass exists to undo;
making the same choice in the other direction would be the same mistake wearing
the other coat.
