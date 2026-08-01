**Proof: CSS color-matrix `filter` functions as REAL per-pixel transforms via
our own wgpu color-matrix pass over Vello.**

Renders one colourful reference element (eight vertical colour bars) beside six
copies, each wrapped in a `ColorMatrix` layer —
`PushLayer(SceneLayer::with_clip(bounds).add_effect(color_matrix(M)))` …
`PopLayer` — exactly the command bracket the scene coordinator emits when it
folds an element's color-op `filter` chain into one 4×5 matrix. The renderer's
owned color-matrix wgpu pass applies each `M`; the read-back PNG
(`target/color_filter_proof.png`) shows each transform:

- `grayscale(1)`  — the bars collapse to their luminance grays,
- `brightness(1.5)` — everything lightens,
- `saturate(2)`   — the colours intensify,
- `invert(1)`     — every colour negates (red→cyan, white→black, …),
- `hue-rotate(120deg)` — hues rotate a third of the wheel (red→green→blue),
- `sepia(1)`      — the image warms to brown tones.

Vello 0.9 has no color filters, so this drives the renderer half of the slice
end to end (sub-scene extraction → the owned color-matrix fragment pass →
composite). The matrices here are the spec-exact folds the coordinator produces
(its folding is separately unit-tested); this hand-built list is byte-for-byte
the structure it emits for these single-function filters.

# Environment note

Needs a usable GPU adapter. With none it prints a diagnostic and exits cleanly
(no panic); it never fabricates a device.

Run it:
`cargo run -p obscur-kit --example color_filter_proof --features render-vello`
