**Proof: CSS `repeating-linear-gradient()` tiled as REAL stripes via Vello.**

Drives the *whole* slice end-to-end: authors one CSS rule
`background-image: repeating-linear-gradient(45deg, #2b2a27 0 6px, #faf9f6 6px
12px)` on a single box, compiles it through the real `obscur-css` frontend,
runs one frame through [`UiRuntime`] (so the scene coordinator resolves the
one-period gradient axis + `ExtendMode::Repeat`), and renders it through
headless Vello/wgpu into `target/repeating_gradient_proof.png`.

The 12px period tiles across the box at 45°, so the read-back PNG shows a field
of alternating dark/light diagonal stripes — NOT a single ramp. On the CPU
backend the same gradient would flatten to its first (dark) stop per the
GPU-class-effect policy; the faithful stripes are the Vello proof.

# Environment note

Needs a usable GPU adapter. With none it prints a diagnostic and exits cleanly
(no panic); it never fabricates a device.

Run it:
`cargo run -p obscur-kit --example repeating_gradient_proof --features render-vello`
