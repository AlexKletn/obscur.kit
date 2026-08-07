Lowers this canvas into its [`ViewNode::Element`], emitting the reserved local
name `canvas` (the element's CSS *type*, spec §4.13) followed by its width and
height as float properties.

The name is what makes any `canvas { … }` rule reach a `Canvas::new(…)` node:
the matcher matches a type selector against a node's local name, an authored
`<canvas>` tag already contributed one, and that tag binds to this very
component id. Without it the drawn controls' painter box — the `<canvas>` child
`Progress::contents` and `Slider::contents` build — was unreachable from every
stylesheet in the system, author's and user agent's alike, and the UA sheet had
to size it as the anonymous `progress > *, slider > *`.
