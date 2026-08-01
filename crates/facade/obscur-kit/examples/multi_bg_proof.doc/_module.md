**Proof: multiple `background-image` layers (a comma list) via Vello.**

Drives the *whole* slice end-to-end: authors one CSS rule whose
`background-image` is a **two-layer** comma list — two crossing
`repeating-linear-gradient()`s — over a solid `background-color`, compiles it
through the real `obscur-css` frontend (which splits the top-level comma into
two layers), runs one frame through [`UiRuntime`] (so the scene coordinator
paints both layers back-to-front over the color fill), and renders it through
headless Vello/wgpu into `target/multi_bg_proof.png`.

The first layer (default `to bottom`) tiles 1px lines every 24px along the
vertical axis → HORIZONTAL grid lines; the second (`90deg`) tiles the same 1px
lines along the horizontal axis → VERTICAL grid lines. Because the lines sit on
`transparent` gaps, the bottom layer shows through the top layer's gaps, so the
read-back PNG shows a **crossing graph-paper grid** (both line sets), not just
one — the multi-layer proof. On the CPU backend each repeating gradient would
flatten to its first stop per the GPU-class-effect policy; the faithful grid is
the Vello proof.

# Environment note

Needs a usable GPU adapter. With none it prints a diagnostic and exits cleanly
(no panic); it never fabricates a device.

Run it:
`cargo run -p obscur-kit --example multi_bg_proof --features render-vello`
