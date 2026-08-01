**Headless imported-UI-Kit → PNG.**

Renders the shared [`uikit`] scene — the theme-switcher bar over the component
gallery — through the **software** [`CpuRenderer`] (no display, no GPU) and
writes the frame to `target/uikit_<slug>_cpu.png`, a real viewable artifact.

The scene is *theme-agnostic*: [`uikit::view`] uses only semantic classes and
[`uikit::themes`] supplies the appearance. The active theme is selected by the
`UIKIT_THEME` env var (a 0-based index into [`uikit::themes`], default
[`uikit::MATERIAL_INDEX`]); e.g. `UIKIT_THEME=4` renders the **Skeuomorphic**
theme to `target/uikit_skeuo_cpu.png`.

This is deliberately the **honest CPU approximation**: GPU-class effects degrade
per the project's effect-split policy (a multi-layer `box-shadow` paints as flat
offset rects — inset gloss lines and cast shadows both flatten; a
`linear-gradient` paints only its FIRST stop, so the Skeuomorphic theme's glossy
ramps read as flat light fills). Layout, text, borders, radius, and color render
faithfully. Compare with `uikit_vello_png.rs`, the GPU render where the same
gradients and shadows are real.

Run it (Skeuomorphic):
`UIKIT_THEME=4 cargo run -p obscur-kit --example uikit_png \
  --features "render-cpu resource-fs resource-decode text-harfrust text-unicode"`
