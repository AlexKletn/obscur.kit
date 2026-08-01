The `mix-blend-mode` property (a `BlendMode` keyword; NOT inherited).
Initial `normal`. A value other than `normal` makes the element a group that
composites onto the backdrop with that separable blend mode (compositing-1
§5): the scene coordinator brackets the element's subtree with a
`PushLayer` carrying the blend mode. Keyword handles live in
[`crate::property::layout_keyword`].
