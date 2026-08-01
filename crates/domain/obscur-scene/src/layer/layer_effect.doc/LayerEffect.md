An effect applied to a layer's composited content (spec §5.1).

A `LayerEffect` is a backend-independent descriptor of a post-composition
operation: uniform opacity, a Gaussian-style blur, a blend mode against
the backdrop, or a color-matrix transform. It references no GPU or renderer
types. Shadows are a later addition.
