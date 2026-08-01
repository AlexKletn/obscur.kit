How a shape is painted (spec §5.1): its interior filled with a [`Brush`], or
its outline drawn with a [`Stroke`].

`Paint` describes only the color source and geometry of the mark. A shape's
final on-screen appearance is this paint combined with the blend mode and
opacity applied by its enclosing layer.
