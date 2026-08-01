**Proof: an `inset` `box-shadow` renders INSIDE the box (headless CPU).**

Builds a display list of two light boxes over a mid-grey page, each carrying an
inset `box-shadow` exactly as the scene coordinator emits it — a background fill
(the light box) followed by an inset `DrawShadow` painted OVER the background:

- Left:  `box-shadow: 0 0 0 3px #2b2a27 inset` — a crisp inner RING hugging the
  box edge (blur 0, spread 3).
- Right: `box-shadow: inset 0 4px 8px rgba(0,0,0,.4)` — a soft inner shadow along
  the TOP edge (offset down, blur 8; the software backend approximates the blur
  as a hard frame, thicker on the side opposite the offset).

It rasterizes through the software [`CpuRenderer`] (no GPU) and writes
`target/inset_shadow_proof.png`. Before writing, it asserts for the ring box that
the box EDGE is inked, the box CENTER is clear (a ring, not a filled blob), and a
pixel just OUTSIDE the box is the untouched page grey — i.e. the inset shadow is
confined to the box and never bleeds out.

Run it:
`cargo run -p obscur-kit --example inset_shadow_proof --features render-cpu`
