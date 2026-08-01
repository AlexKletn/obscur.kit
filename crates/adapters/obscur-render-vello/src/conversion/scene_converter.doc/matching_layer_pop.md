The index of the `PopLayer`/`PopClip` matching the `PushLayer` at `start`,
counting nested `Push*`/`Pop*` of both clip and layer kinds. Returns
`commands.len()` for an unbalanced list (the rest is treated as the subtree).
