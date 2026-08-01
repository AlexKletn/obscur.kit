The fontconfig pattern naming a CSS generic family.

fontconfig's own config defines `serif` / `sans-serif` / `monospace` aliases;
`cursive` and `fantasy` are CSS generics fontconfig also recognises. `SystemUi`
has no fontconfig alias, so it asks for the same face `sans-serif` does —
which is what the freedesktop stack means by "the UI font".
