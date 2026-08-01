**Proof: canonical CSS gradient GEOMETRY (corner direction, radial extent
keywords, ellipse) via Vello.**

Drives three stacked WIDE (non-square, 480 × 120) boxes end-to-end through the
real `obscur-css` frontend + [`UiRuntime`] scene coordinator, then renders them
through headless Vello/wgpu into `target/gradient_geometry_proof.png`:

1. `background-image: linear-gradient(to top right, #000, #fff)` — the gradient
   line runs to the box's actual top-right corner. On the wide box its direction
   is `normalize(h, −w) = normalize(120, −480)` (spec §3.1) — a *shallow* angle
   (≈14° above horizontal), NOT a fixed 45°. The dark→light ramp therefore runs
   nearly horizontally, tilting slightly up-right.
2. `radial-gradient(closest-side circle, …)` — a true circle whose radius is the
   distance to the closest side (60px, the half-height), so a tight centered
   disc that fades out well before the box's left/right edges.
3. `radial-gradient(ellipse farthest-corner, …)` — an ellipse reaching the
   farthest corner, radii `(dx_far·√2, dy_far·√2) = (240√2, 60√2) ≈ (339, 85)`:
   a horizontally STRETCHED oval (rx ≫ ry), not a circle.

On the CPU backend every gradient flattens to its first stop per the GPU-class
effect policy; the faithful geometry is the Vello proof.

# Environment note

Needs a usable GPU adapter. With none it prints a diagnostic and exits cleanly.

Run it:
`cargo run -p obscur-kit --example gradient_geometry_proof --features render-vello`
