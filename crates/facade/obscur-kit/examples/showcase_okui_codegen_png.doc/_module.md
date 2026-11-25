**`.okui` → generated Rust → View → headless CPU → PNG.**

The codegen counterpart of [`showcase_okui_png`](../showcase_okui_png.rs).
Where that example parses, compiles and *bridges* `showcase.okui` into a
`View` at runtime, this one consumes a `View` that was compiled **at build
time**: `build.rs` runs the frontend pipeline over `showcase.okui` and emits a
`pub fn showcase_view() -> View` into `OUT_DIR`, which this example
`include!`s. No parser, compiler, or bridge runs here — the `.okui` is
*ordinary code* by the time this binary starts.

Because the generated `View` resolves names through the shared
`name_binding` registry, real widgets (`<Button>`) and `<img src>` carry
their true `ComponentId`s and reserved symbols — the two "honest bridge gaps"
called out in `showcase_okui_png` are closed on this path.

Run it:
`cargo run -p obscur-kit --example showcase_okui_codegen_png \
  --features "render-cpu resource-fs resource-decode text-harfrust text-unicode"`
