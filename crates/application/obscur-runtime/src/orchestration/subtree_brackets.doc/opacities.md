A CSS `opacity` below `1.0`: the subtree composites as ONE group through a
`PushLayer(SceneLayer::with_opacity(a))` … `PopLayer`, nested *inside* any
transform and *outside* the rounded clip.
