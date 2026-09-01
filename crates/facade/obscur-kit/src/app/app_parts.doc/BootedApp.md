A runtime with everything installed that does not need a surface, plus the
pieces the runner still has to place itself.

The split is exactly "what a surface is needed for": stylesheets, commands,
virtual lists, images and the executor are all installable against a bare
runtime, and the text stack is not — it needs a renderer to register fonts
into, and the renderer needs a surface.
