**Proof: CSS `backdrop-filter: blur()` as REAL frosted-glass / acrylic.**

Renders a busy hard-edged checkerboard BACKDROP filling the viewport through
headless Vello/wgpu, then lays a semi-transparent rounded panel on top carrying
`backdrop-filter: blur(10px)` — a `BackdropFilter { bounds, radii, blur }`
command emitted BEFORE the panel's own translucent fill, exactly what the scene
coordinator produces. The renderer's segmented compositing frosts the backdrop
*under the panel only*: the read-back PNG (`target/backdrop_proof.png`) shows the
checkerboard visibly blurred inside the rounded panel (with the tint reading as
acrylic) while the surrounding backdrop stays crisp and the corners stay sharp.

This drives the whole renderer half of the slice (display-list segmentation →
per-segment offscreen → composite onto the accumulating frame → in-place region
blur masked to the rounded radii). The CSS lowering and the coordinator's
`BackdropFilter` emission are covered by unit tests; this hand-built display list
is byte-for-byte the structure those produce.

# Environment note

Needs a usable GPU adapter. With none it prints a diagnostic and exits cleanly
(no panic); it never fabricates a device.

Run it:
`cargo run -p obscur-kit --example backdrop_proof --features render-vello`
