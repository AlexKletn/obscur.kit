Builds the real [`HarfrustShaper`](crate::text_harfrust::HarfrustShaper)
wired to a single host font from its raw `bytes`.

Install the result on a runtime with
[`UiRuntime::set_text_shaper`](crate::runtime::UiRuntime::set_text_shaper),
using the **same** bytes handed to the renderer's `set_font`, so the
shaper's emitted glyph indices align with the font the renderer rasterizes.
The shaper is returned boxed as the domain `TextShaper` port, so callers
depend only on the port, not the concrete adapter.
