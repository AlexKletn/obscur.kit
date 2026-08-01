**Proof: 2D `transform` interpolation via matrix decomposition
(CSS Transforms §14).**

Takes the two endpoints of a `transform: rotate(0deg) → rotate(90deg)`
transition as [`MotionValue::Transform`] affines and blends them at the
midpoint `t = 0.5` through the motion engine. The correct answer is
`rotate(45°)` **at full scale** — `[cos45, sin45, −sin45, cos45, 0, 0]` ≈
`[0.707, 0.707, −0.707, 0.707, 0, 0]` — NOT the naive component-wise matrix
lerp `[0.5, 0.5, −0.5, 0.5]`, whose column length `hypot(0.5, 0.5) ≈ 0.707`
would shrink the box 30% and shear it.

It drives the interpolated affine through the REAL paint path: a
`PushTransform(translate(C) · matrix · translate(−C))` bracket rotates a
100×100 square about its centre, rasterized headless through the software
[`CpuRenderer`] to `target/transform_interp_proof.png`. The square becomes a
**diamond** whose top vertex sits a half-diagonal `50·√2 ≈ 70.71px` above the
centre — the example self-asserts that measured extent (a rotate that
*preserved scale*), which a shrunk/sheared naive lerp could never reach.

This is the last item of the math-port campaign; the interpolation math lives
in `obscur_motion::value` (decompose → interpolate → recompose).

Run it:
`cargo run -p obscur-kit --example transform_interp_proof --features render-cpu`
