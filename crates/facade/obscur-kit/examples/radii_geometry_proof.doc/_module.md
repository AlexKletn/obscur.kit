**Proof: canonical CSS radii geometry (headless CPU).**

Renders two panels that exercise the two fixes and writes
`target/radii_geometry_proof.png`:

- **Left — border-radius overlapping-radius clamp (css-backgrounds §4.5).** A
  wide-short `200×40` box with `border-radius: 30px` on all four corners. Each
  corner (30) is below `min(w,h)/2 = 20`? No — the LEFT/RIGHT sides sum to
  `30+30 = 60 > 40` (the height), so the spec's single uniform scale
  `f = 40/60` shrinks ALL four corners to `20px`, preserving proportions. The
  panel fills the box with the §4.5-normalized radii; the corners fit and never
  overflow the box.
- **Right — box-shadow spread → radii (css-backgrounds §6.1.1).** A rounded
  `border-radius: 10px` box with an outset `box-shadow` of `spread: 8px`. The
  shadow's corner radii are the box radii grown by `+spread` → `18px`, so the
  shadow's rounded corners are visibly LARGER than the box's and stay
  concentric with them.

It rasterizes through the software [`CpuRenderer`] (no GPU). Before writing, it
asserts (left) the normalized radius is `20px` and the box corner is rounded away
while the mid-edges reach the box bounds, and (right) that the shadow extends
beyond the box corner (a larger, concentric round).

Run it:
`cargo run -p obscur-kit --example radii_geometry_proof --features render-cpu`
