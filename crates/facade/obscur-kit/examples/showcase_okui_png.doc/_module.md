**`.okui` DSL → View bridge → headless CPU → PNG.**

This example drives the *real* `.okui` frontend pipeline end-to-end and
rasterizes the result with the software backend — no display, no GPU:

1. **Parse** an authored [`showcase.okui`](../showcase.okui) document with the
   lossless [`Parser`](obscur_kit::okui_syntax::parser::Parser).
2. **Compile** it through the [`Compiler`](obscur_kit::okui_compiler::compile::Compiler)
   (name resolution → validation against the built-in contracts → lowering to
   UI IR). Diagnostics are printed; the produced IR is still rendered.
3. **Bridge** the compiled `UiIrDocument` into a runtime desired
   [`View`](obscur_kit::view::view::View) with
   [`UiIrViewBridge::document_to_view`](obscur_kit::okui_compiler::bridge::UiIrViewBridge).
4. **Render** that `View` + an authored stylesheet + a staged image asset
   through the [`UiRuntime`] and the [`CpuRenderer`], writing
   `target/showcase_okui.png`.

# What renders *from `.okui`* today

The bridge maps each element's `class` / `id` / `type` attributes onto the
reserved CSS selector symbols, so **every CSS-driven visual matches on the
`.okui`-authored classes/types**: solid + gradient (linear / radial / conic)
backgrounds, border + radius, box-shadow, `filter: drop-shadow`, `opacity`,
`transform`, and `background-image: url(...)`. Structure/nesting (grid + flex
layout) and shaped **text** flow through too. GPU-class effects degrade to the
documented CPU approximation (a gradient paints its first stop; a shadow a flat
offset rect), exactly as in `showcase_png.rs`.

# The honest bridge gaps (NOT rendered from `.okui` yet)

- **Real widgets.** `<Button>` / `<Input>` / `<Checkbox>` lower to a
  placeholder `ComponentId` (a hash of the tag), not the real widget
  `COMPONENT_ID`, so they render as bare classed boxes, not the widgets.
- **`<img src>`.** The bridge hashes arbitrary attributes, so an image
  element's `src` never reaches the reserved `SRC` symbol the runtime resolves
  to pixels. A `background-image: url(...)` card (pure CSS) IS shown instead.

Run it:
`cargo run -p obscur-kit --example showcase_okui_png \
  --features "render-cpu resource-fs resource-decode text-harfrust text-unicode"`
