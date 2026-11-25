The unbounded outline itself, borrowed: the shape a layer with no clip and a
layer clipped by a [`Mask`](obscur_scene::clip::Mask) both resolve to.

Built once for the process and handed out by reference. This is not a cache and
needs no invalidation: it is a function of no input at all, so there is no key
it could answer wrongly and no event that could make it stale — only the
allocation stops it from being a `const`.

Use this wherever the outline is only read (`Scene::push_layer` takes it by
reference); [`ClipConverter::unbounded`] clones it for callers that need to own
one.
