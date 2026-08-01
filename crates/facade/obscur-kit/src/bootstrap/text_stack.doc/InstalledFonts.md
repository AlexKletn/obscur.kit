What [`install_text_stack`](super::KitBootstrap::install_text_stack) hands
back: the scene font handle assigned to each installed face, paired with the
bytes behind it.

# Why the install cannot finish the job

The runtime resolves and shapes; it never rasterizes. The renderer does, and it
is not the runtime's to reach — a host owns it (and its GPU device). So the
shaper's emitted glyph **indices** are only meaningful to a renderer that has
been given the *same* bytes under the *same* handle. Feed this to the renderer
before the first frame:

```ignore
let fonts = boot.install_text_stack(&mut runtime, &font_set);
for (handle, bytes) in fonts.iter() {
    renderer.register_font(handle, bytes.to_vec());
}
```

The two failure modes differ, and neither is silent-but-fine. Skip the loop and
the renderer has no bytes under the handle, so it falls back to drawing **box
marks** (`draw_glyph_boxes` in the CPU backend) — visibly tofu. Register
*different* bytes under the same handle and it is worse: the halves disagree
about what glyph `7` is, and real-looking but **wrong outlines** paint. Nothing
here can force the hand-off; this table is what makes it one loop.
