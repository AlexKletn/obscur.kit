**Proof: a real `overflow-y: auto` vertical scroll container — clip, scroll
offset, scrollbar, and a scrolled hit-test.**

Reimplemented from **CSS Overflow Module Level 3** (§overflow, §scroll-container,
§scrollport). A 200×120 `.scroller` with `overflow: auto` holds six 40-px rows
(240 px of content), so it is a scroll container. This drives the REAL pipeline
through the public facade on a live [`UiRuntime`]: reconcile → style → layout →
compose, where the scene coordinator brackets the scroller's descendants with a
`PushTransform(translate(0, −scroll_top))` inside its overflow clip and paints a
vertical scrollbar. Two panels are rasterized headless through the software
[`CpuRenderer`] (no GPU) to `target/scroll_proof.png`:

- **Panel 1 — Idle.** `scroll_top = 0`: rows 0–2 fill the scrollport; rows 3–5
  are CLIPPED away (they do not paint below the box). The scrollbar thumb sits
  at the top.
- **Panel 2 — Scrolled.** After [`UiRuntime::scroll_by`]`(scroller, 80)` the
  content is translated up 80 px: row 2 now paints at the top and rows 0–1 have
  scrolled out. The thumb has moved down.

Before writing the image it machine-checks: (a) row 3 does NOT paint below the
scrollport (the clip bites); (b) the top-of-scrollport pixel is row 0 idle and
row 2 scrolled (the content scrolled up by exactly one 80-px step / two rows);
(c) a scrollbar thumb paints in the right gutter and moves down when scrolled;
and (d) a pointer at the same screen point hit-tests row 0 idle but row 2
scrolled — the pointer mapping composes the container's `scroll_top`.

Run it:
`cargo run -p obscur-kit --example scroll_proof --features render-cpu`
