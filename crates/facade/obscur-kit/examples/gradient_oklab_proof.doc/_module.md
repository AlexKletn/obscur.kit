**Proof: CSS gradients interpolate in Oklab (CSS Images 4 default) via Vello.**

Per **CSS Images Module Level 4 §3.4**, a gradient with no authored
`in <color-space>` interpolates its color stops in **Oklab** — the default
rectangular interpolation space (CSS Color 4 §12). Today's fix sets
`Gradient::interpolation_cs = Oklab` on the peniko builders in
`brush_converter.rs`; this example proves Vello actually *honors* that field
during rasterization.

It drives two horizontal 256 × 64 bands end-to-end through the real
`obscur-css` frontend + [`UiRuntime`], then renders them headless through
Vello/wgpu into `target/gradient_oklab_proof.png`:

1. `linear-gradient(to right, red, green)` — the naive sRGB-gamma midpoint is
   a dark muddy olive (~`(128, 64, 0)`-ish once premultiplied over sRGB). The
   **Oklab** midpoint is a distinctly *lighter*, more vivid yellow-green: its
   green channel and overall luminance are meaningfully higher. The example
   **self-asserts** the sampled midpoint green channel clears a threshold that
   the sRGB-gamma mix could never reach — the machine-checked proof that Vello
   interpolated in Oklab.
2. `linear-gradient(to right, blue, yellow)` — rendered for visual comparison.

On the CPU backend every gradient flattens to its first stop per the GPU-class
effect policy; the faithful interpolation is the Vello proof.

# Environment note

Needs a usable GPU adapter. With none it prints a diagnostic and exits cleanly
(never a fabricated device / panic).

Run it:
`cargo run -p obscur-kit --example gradient_oklab_proof --features render-vello`
