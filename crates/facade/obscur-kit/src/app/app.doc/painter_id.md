Registers a canvas painter under the symbol a `<canvas>` node names directly.

A node's own painter property outranks anything a stylesheet says, which is
what keeps a widget's drawing its own: a rule matching a progress bar cannot
replace the routine that draws it with one that draws a clock. So a canvas
built in Rust —

```ignore
Canvas::new(240.0, 80.0).named(SPARKLINE).into_view_node()
```

— is reached by this method and not by [`painter`], which registers under the
CSS-visible *name*. The two keys are separate namespaces; a painter registered
under one is not found by the other.

Registering under a shipped widget's symbol ([`Progress::PAINTER`],
[`Slider::PAINTER`]) replaces the built-in for the whole application, which is
how an application changes what a progress bar looks like beyond its accent.
It wins over the built-in because the app's registrations are applied after
the bootstrap's.

[`painter`]: App::painter
[`Progress::PAINTER`]: crate::widgets::progress::Progress::PAINTER
[`Slider::PAINTER`]: crate::widgets::slider::Slider::PAINTER
