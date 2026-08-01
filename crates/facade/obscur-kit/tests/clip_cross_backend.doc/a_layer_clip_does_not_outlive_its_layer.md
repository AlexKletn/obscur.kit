A layer's clip must stop confining drawing once the layer is popped.

The other direction of honoring a layer clip, and the one a single-bracket
scene cannot see: a clip that is pushed but never popped confines everything
drawn afterwards too, which is just as wrong as ignoring it and would let the
push half of the fix pass this file alone. The scene draws *outside* the
layer's clip after `PopLayer`; that paint must survive.
