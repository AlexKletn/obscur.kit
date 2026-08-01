A layer's clip reaches Vello's real `push_layer`, carrying the geometry the
domain asked for.

The other layer test asks `ClipConverter` what Vello *would* resolve, which
proves the rule and not the plumbing: a `push_layer` that dropped `clip()` on
the floor — the CPU backend's exact bug — would pass it. This drives the public
`SceneConverter::convert` and reads the clip back out of `Scene::encoding()`,
so the assertion is that the converter really encoded it.

No GPU: `encoding()` is the scene's CPU-side buffer, populated by `push_layer`
before anything is submitted. This is the one part of the Vello comparison that
does not need an adapter.
