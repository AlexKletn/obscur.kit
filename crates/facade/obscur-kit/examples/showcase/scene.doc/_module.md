The **shared capability-showcase** scene, authored once and rendered by both
the headless CPU→PNG example (`showcase_png.rs`) and the windowed Vello
example. It is deliberately **backend-pure**: it names only the toolkit's
view / widget / primitive / CSS authoring surfaces (`obscur_kit::*`) and never
a renderer, window, or GPU type, so the exact same view + stylesheet feed a
software rasterizer and a GPU surface unchanged.

# What it exercises

A titled "gallery" laid out with a **grid** of cards over a **flex** toolbar
row, each card demonstrating one shipped capability:

- layout: a `display: grid` gallery (multiple tracks) + a `display: flex` row;
- backgrounds: `background-color`, `linear-gradient`, `radial-gradient`,
  `conic-gradient`;
- borders: `border` + `border-radius`, and a thick colored border;
- effects: `box-shadow`, `filter: drop-shadow(...)`, group `opacity: 0.5`,
  `transform: rotate()/scale()`;
- images: a `background-image: url(...)` card and an `<img>` [`Image`]
  primitive with `object-fit: contain`;
- text: headings + paragraphs at several sizes / weights (a mixed-script
  sample too);
- widgets: a [`Button`], a [`Checkbox`], and an [`Input`] carrying a value;
- a state-driven counter value rendered as text.

The gradient / shadow / blur effects are GPU-class: on the CPU backend they
degrade to a documented approximation (a gradient paints its first stop; a
shadow paints a flat offset rect) per the project's CPU/GPU effect-split
policy — the layout, text, images, borders, radius, opacity, and transform
geometry all render faithfully. The windowed Vello example shows the effects
at full fidelity from the very same module.
