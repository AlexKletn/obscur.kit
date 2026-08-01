Proves the demo's `#app` page is a genuine vertical scroll container: it clips +
scrolls its content (the `.wf-col` sheet) via `scroll_by`, exactly as the
windowed driver's mouse wheel does, and paints a scrollbar. Rasterizes an
idle|scrolled pair of the window-sized sheet to `target/wireframe_scroll_proof.png`.
