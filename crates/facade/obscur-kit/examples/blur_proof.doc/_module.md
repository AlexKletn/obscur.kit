**Proof: CSS `filter: blur()` as a REAL gaussian via our own wgpu pass.**

Renders two identical hard-edged checkerboards side by side through headless
Vello/wgpu: the LEFT one is drawn straight (sharp), the RIGHT one is wrapped in
a blur layer — `PushLayer(SceneLayer::with_clip(bounds).add_effect(blur(6)))` …
`PopLayer` — exactly the command bracket the scene coordinator emits for
`filter: blur(6px)`. The renderer's owned separable-gaussian pass blurs the
right panel; the read-back PNG (`target/blur_proof.png`) shows the right
checkerboard visibly softened while the left stays crisp.

This drives the whole renderer half of the slice (sub-scene extraction in the
converter → H+V gaussian pipelines → premultiplied composite). The CSS lowering
and the coordinator's blur bracket are covered by unit tests; this hand-built
display list is byte-for-byte the structure those produce.

# Environment note

Needs a usable GPU adapter. With none it prints a diagnostic and exits cleanly
(no panic); it never fabricates a device.

Run it:
`cargo run -p obscur-kit --example blur_proof --features render-vello`
