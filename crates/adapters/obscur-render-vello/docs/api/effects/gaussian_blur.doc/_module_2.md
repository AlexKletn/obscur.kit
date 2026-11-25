Deterministic CPU port of the `blur_fs` decode → blur → encode pipeline,
validating (without a GPU) that convolving in **linear light** — per the
Filter Effects Module default `color-interpolation-filters: linearRGB` — keeps
a bright/dark boundary brighter than a naive gamma-sRGB gaussian, and that a
flat opaque region round-trips the transfer unchanged.

These mirror the exact WGSL helpers/formulas in `gaussian_blur.wgsl`; the GPU
render `examples/blur_linear_light_proof.rs` proves the shader itself.
