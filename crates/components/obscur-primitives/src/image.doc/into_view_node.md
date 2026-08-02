Lowers this image into its [`ViewNode::Element`], emitting the reserved local
name `img` (the element's CSS *type*, spec §4.13), the source property and,
when set, the alt property.

The name is what makes the UA stylesheet's `img { … }` rule and the
accessibility projection's `SemanticRole::Image` reach an `Image::new(…)` node:
both read a local name, both already reached an authored `<img>` tag, and that
tag binds to this very component id.
