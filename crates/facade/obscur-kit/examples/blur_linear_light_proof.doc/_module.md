**Proof: the gaussian blur convolves in LINEAR light, not gamma-sRGB.**

Per the Filter Effects Module Level 1, the initial value of
`color-interpolation-filters` is `linearRGB`, so `feGaussianBlur` — what CSS
`filter: blur()` / `backdrop-filter: blur()` lower to — must convolve in linear
light. A gamma-space gaussian darkens/muddies bright-on-dark and colored edges;
a linear-light one keeps saturated edges bright and clean.

This renders, headless through Vello/wgpu, a saturated GREEN box (`#00ff00`)
centered on a RED field (`#ff0000`), the whole thing wrapped in a blur layer —
`PushLayer(SceneLayer::with_clip(bounds).add_effect(blur(R)))` … `PopLayer`, the
exact bracket the coordinator emits for `filter: blur(Rpx)`. The green/red edge
is an INTERNAL edge inside the blurred subtree (both colors in one texture,
fully opaque), so the transition band isolates the color-space difference.

It writes `target/blur_linear_light_proof.png` and self-asserts:
  * at the green↔red crossover (where the blurred R and G channels are equal),
    that value is well ABOVE the gamma-space average of 128 — a gamma blur would
    land near ~128, linear light lands near ~182 (= sRGB-encode of the linear
    midpoint 0.5);
  * flat interior regions round-trip the transfer: deep green stays ~`#00ff00`
    and deep red stays ~`#ff0000` (blur of a constant is that constant).

# Environment note

Needs a usable GPU adapter. With none it prints a diagnostic and exits cleanly
(no panic); it never fabricates a device.

Run it:
`cargo run -p obscur-kit --example blur_linear_light_proof --features render-vello`
