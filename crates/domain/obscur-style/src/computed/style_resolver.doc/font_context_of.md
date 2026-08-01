The style a [`ChMetrics`](super::ChMetrics) port should measure to get the **parent's**
metrics, per CSS Values 4 §6.1.2.

For a node with a parent this is simply the parent's computed style, which
already carries its resolved `font-size` and face.

For the **root**, §6.1.2 says a font-relative unit on a `font-*` property
resolves *"against the computed metrics corresponding to the initial values
of the font and line-height properties, if the element has no parent"* — so
the answer is not "unmeasurable", it is "measure the initial font". That is
a style carrying the initial `font-size` and no `font-family`, which is
exactly what an implementor reads as the default face.
