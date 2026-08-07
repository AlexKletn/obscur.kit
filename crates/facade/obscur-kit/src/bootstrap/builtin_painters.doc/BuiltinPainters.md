The registration set for the shipped widgets' canvas painters.

Holds the one thing the painters cannot read for themselves: the accent
colour. `CanvasPaint` carries a node's properties and the elapsed time, not
its computed style, so the brush a progress bar fills with and a slider's
thumb is filled with is decided *here*, once, at registration — not per
node, and not from CSS.

That is a real limit and worth stating plainly: `-obscur-painter` binds a
routine, and no CSS property parameterises one. A host that wants a
different accent installs the set with [`with_accent`]; a host that wants
per-node colour registers its own painter under
[`Progress::PAINTER`] / [`Slider::PAINTER`] through
[`App::painter_id`], which replaces the built-in.

[`with_accent`]: BuiltinPainters::with_accent
[`Progress::PAINTER`]: crate::widgets::progress::Progress::PAINTER
[`Slider::PAINTER`]: crate::widgets::slider::Slider::PAINTER
[`App::painter_id`]: crate::app::App::painter_id
