Runs one raster load to completion: the densest DPI variant in
`candidates` that resolves, loaded and decoded into a [`SceneImage`], as
the outcome of the job started for `handle` at `version` and keyed by
`key` (spec §5.15 load → §5.16 decode).

Every failure is carried *inside* the returned value rather than raised:
the job's business is to answer, and only the hub may rule on what that
answer is worth.
